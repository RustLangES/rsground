mod common;

use awc::http::StatusCode;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};

async fn login_as(guest_name: &str) -> (String, String) {
    request!([POST] "/auth/guest"
        send { "guest_name": guest_name },
        expect OK,
        json,
        tee_ref
        [ get "jwt", as string ]
        [ get "id", as string ]
    )
}

async fn create_project(token: &str) -> String {
    request!([POST] "/create/test_project"
        as token,
        send,
        expect CREATED,
        json,
        get "id",
        as string
    )
}

/// Test que simula el flujo completo con dos usuarios:
/// 1. Inician sesión como "guest" y "owner"
/// 2. Se conectan vía WebSocket y reciben su user_id
/// 3. Con el token de owner se crea un proyecto privado con contraseña
/// 4. El usuario guest solicita unirse al proyecto con acceso de editor
/// 5. El owner aprueba la solicitud con grant_editor
/// 6. El usuario guest realiza varias operaciones: insert, delete, sync y consulta de archivos
#[actix_rt::test]
async fn test_flow_two_users() {
    // --- 1. Log in for both users ---
    let (owner, owner_id) = login_as("owner").await;
    let (guest, guest_id) = login_as("guest").await;

    // --- 2. Create project as "owner" ---
    let project_id = create_project(&owner).await;

    // --- 3. Connect both websockets ---
    let mut owner_ws = ws!(connect owner, &project_id);

    // Owner handshake
    {
        let (_, users) = ws!(recv owner_ws, "welcome", [ get "users", as object ]);
        assert!(users.contains_key(&owner_id), "Self should be included")
    }

    // Owner get notified about its own connection
    _ = ws!(recv owner_ws, "user_connected", [ get "user_id", as string, eq owner_id ]);

    let mut guest_ws = ws!(connect guest, &project_id);

    // Guest handshake
    {
        let (_, users) = ws!(recv guest_ws, "welcome", [ get "users", as object ]);
        assert!(users.contains_key(&guest_id), "Self should be included")
    }

    // Get notified about guest connection
    _ = ws!(recv guest_ws, "user_connected", [ get "user_id", as string, eq guest_id ]);
    _ = ws!(recv owner_ws, "user_connected", [ get "user_id", as string, eq guest_id ]);

    // --- 4. Gives editor to guest ---
    _ = ws!(send owner_ws, "permit_access" {
        "user_id": guest_id,
        "access": "editor"
    });

    // Access update
    _ = ws!(recv owner_ws, "update_access", [ get "user_id", as string, eq guest_id ] [ get "access", as string, eq "editor" ]);
    _ = ws!(recv guest_ws, "update_access", [ get "user_id", as string, eq guest_id ] [ get "access", as string, eq "editor" ]);

    // --- 4. Inserts text in "test" file ---
    _ = ws!(send guest_ws, "file_create" {
        "file": "test"
    });

    _ = ws!(send guest_ws, "sync" {
        "revision": 0,
        "file": "test",
        "actions": [{ "kind": "insertion", "from": 0, "text": "hello world", "owner": guest_id }],
    });

    _ = ws!(recv guest_ws, "sync", [ get "actions", as array, dbg ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 1 ]);
    _ = ws!(recv owner_ws, "sync", [ get "actions", as array, dbg ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 1 ]);

    // --- 7. Guest inserta texto en "src/wello.txt" ---
    let insert_wello_msg = json!({
        "action": "insert",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "file": "src/wello.txt",
        "pos": 0,
        "text": "hello mundo"
    });
    guest_ws
        .send(awc::ws::Message::Text(insert_wello_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "update",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
                "file": "src/wello.txt",
                "content": "hello mundo"
            })
        );
    } else {
        panic!("No se recibió respuesta al insertar en src/wello.txt");
    }

    // --- 8. Guest inserta texto en "src/tests/tests.txt" ---
    let insert_tests_msg = json!({
        "action": "insert",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "file": "src/tests/tests.txt",
        "pos": 0,
        "text": "tests mundo"
    });
    guest_ws
        .send(awc::ws::Message::Text(insert_tests_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "update",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
                "file": "src/tests/tests.txt",
                "content": "tests mundo"
            })
        );
    } else {
        panic!("No se recibió respuesta al insertar en src/tests/tests.txt");
    }

    // --- 9. Guest consulta la estructura de archivos del proyecto ---
    let get_files_msg = json!({
        "action": "get_project_files",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded"
    });
    guest_ws
        .send(awc::ws::Message::Text(get_files_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "project_files",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
                "files": {
                    "src": {
                        "wello.txt": null,
                        "tests": {
                            "tests.txt": null
                        }
                    },
                    "documento.txt": null
                }
            })
        );
    } else {
        panic!("No se recibió respuesta a get_project_files");
    }

    // --- 10. Guest borra parte del texto en "documento.txt" ---
    let delete_msg = json!({
        "action": "delete",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "file": "documento.txt",
        "range_start": 0,
        "range_end": 5
    });
    guest_ws
        .send(awc::ws::Message::Text(delete_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "update",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
                "file": "documento.txt",
                "content": "mundo"
            })
        );
    } else {
        panic!("No se recibió respuesta al borrar en documento.txt");
    }

    // --- 11. Guest solicita el histórico (sync) de "documento.txt" ---
    let sync_msg = json!({
        "action": "sync",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "file": "documento.txt",
        "last_timestamp": 0
    });
    guest_ws
        .send(awc::ws::Message::Text(sync_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        // Se espera una respuesta de sync_actions con dos acciones: inserción y eliminación.
        assert_eq!(resp.get("action").unwrap(), "sync_actions");
        assert_eq!(
            resp.get("project_id").unwrap(),
            "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded"
        );
        assert_eq!(resp.get("file").unwrap(), "documento.txt");
        let actions = resp.get("actions").unwrap().as_array().unwrap();
        assert_eq!(actions.len(), 2);

        // Primera acción: inserción
        let insertion = &actions[0];
        assert_eq!(insertion.get("type").unwrap(), "insertion");
        assert_eq!(insertion.get("pos").unwrap(), 0);
        assert_eq!(insertion.get("text").unwrap(), "Hola mundo");
        assert!(insertion.get("timestamp").is_some());

        // Segunda acción: eliminación
        let deletion = &actions[1];
        assert_eq!(deletion.get("type").unwrap(), "deletion");
        assert_eq!(deletion.get("range_start").unwrap(), 0);
        assert_eq!(deletion.get("range_end").unwrap(), 5);
        assert!(deletion.get("timestamp").is_some());
    } else {
        panic!("No se recibió respuesta al hacer sync");
    }
}

fn file_manager() {
    test_flow_two_users()
}

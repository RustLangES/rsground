use actix_rt::test;
use awc::Client;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};

const API_URL: &str = "http://localhost:8080";
const WS_URL: &str = "ws://localhost:8080/ws";

// Función auxiliar para solicitar token de un invitado (guest o owner)
async fn obtener_token_for_user(guest_name: &str) -> String {
    let client = Client::default();
    let mut response = client
        .post(format!("{}/login-guest", API_URL))
        .send_json(&json!({ "guest_name": guest_name }))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    body.get("jwt")
        .expect("Falta la clave 'jwt'")
        .as_str()
        .expect("'jwt' no es una cadena")
        .to_string()
}

/// Test que simula el flujo completo con dos usuarios:
/// 1. Inician sesión como "guest" y "owner"
/// 2. Se conectan vía WebSocket y reciben su user_id
/// 3. Con el token de owner se crea un proyecto privado con contraseña
/// 4. El usuario guest solicita unirse al proyecto con acceso de editor
/// 5. El owner aprueba la solicitud con grant_editor
/// 6. El usuario guest realiza varias operaciones: insert, delete, sync y consulta de archivos
#[test]
async fn test_flow_two_users() {
    // --- 1. Iniciar sesión para ambos usuarios ---
    let guest_token = obtener_token_for_user("guest").await;
    let owner_token = obtener_token_for_user("owner").await;

    // --- 2. Conectarse vía WebSocket para cada usuario ---
    let (guest_response, mut guest_ws) = Client::new()
        .ws(WS_URL)
        .set_header("Authorization", format!("Bearer {}", guest_token))
        .connect()
        .await
        .unwrap();

    let (owner_response, mut owner_ws) = Client::new()
        .ws(WS_URL)
        .set_header("Authorization", format!("Bearer {}", owner_token))
        .connect()
        .await
        .unwrap();

    // Recibir el mensaje de handshake y extraer user_id
    let guest_user_id = if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(resp.get("action").unwrap(), "user_connected");
        resp.get("user_id")
            .expect("Falta user_id en guest handshake")
            .as_str()
            .unwrap()
            .to_string()
    } else {
        panic!("No se recibió handshake para guest");
    };

    let owner_user_id = if let Some(Ok(awc::ws::Frame::Text(txt))) = owner_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(resp.get("action").unwrap(), "user_connected");
        resp.get("user_id")
            .expect("Falta user_id en owner handshake")
            .as_str()
            .unwrap()
            .to_string()
    } else {
        panic!("No se recibió handshake para owner");
    };

    // --- 3. Owner crea el proyecto ---
    let create_project_msg = json!({
        "action": "create_project",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "name": "Proyecto Uno",
        "is_public": false,
        "password": "123"
    });
    owner_ws
        .send(awc::ws::Message::Text(
            create_project_msg.to_string().into(),
        ))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = owner_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "project_created",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded"
            })
        );
    } else {
        panic!("No se recibió respuesta a create_project");
    }

    // --- 4. Guest solicita unirse al proyecto (acceso de editor) ---
    let join_project_msg = json!({
        "action": "join_project",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "access": "editor",
        "password": "123"
    });
    guest_ws
        .send(awc::ws::Message::Text(join_project_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = guest_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(
            resp,
            json!({
                "action": "editor_request_received",
                "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded"
            })
        );
    } else {
        panic!("No se recibió respuesta a join_project");
    }

    // --- 5. Owner concede permisos de editor al usuario guest ---
    let grant_editor_msg = json!({
        "action": "grant_editor",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "user_id": guest_user_id
    });
    owner_ws
        .send(awc::ws::Message::Text(grant_editor_msg.to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(awc::ws::Frame::Text(txt))) = owner_ws.next().await {
        let resp: Value = serde_json::from_slice(&txt).unwrap();
        assert_eq!(resp.get("action").unwrap(), "update");
        assert_eq!(
            resp.get("project_id").unwrap(),
            "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded"
        );
        assert_eq!(resp.get("file").unwrap(), "");
        let expected_content = format!("Usuario {} ahora tiene permisos de editor", guest_user_id);
        // assert_eq!(resp.get("content").unwrap(), expected_content);
        assert_eq!(
            resp.get("content").unwrap().as_str().unwrap(),
            expected_content
        );
    } else {
        panic!("No se recibió respuesta a grant_editor");
    }

    // --- 6. Guest inserta texto en "documento.txt" ---
    let insert_doc_msg = json!({
        "action": "insert",
        "project_id": "948cf4cf-b3d8-4e4a-b9b6-e76e4a1d4ded",
        "file": "documento.txt",
        "pos": 0,
        "text": "Hola mundo"
    });
    guest_ws
        .send(awc::ws::Message::Text(insert_doc_msg.to_string().into()))
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
                "content": "Hola mundo"
            })
        );
    } else {
        panic!("No se recibió respuesta al insertar en documento.txt");
    }

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

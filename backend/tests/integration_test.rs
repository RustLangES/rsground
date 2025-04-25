use core::fmt;

use actix_ws::ProtocolError;
use awc::http::header::TryIntoHeaderPair;
use awc::http::StatusCode;
use awc::ws::Frame;
use awc::{ws, Client, ClientRequest, ClientResponse, SendClientRequest};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Map, Value};

const API_URL: &str = "http://localhost:8080";
const WS_URL: &str = "ws://localhost:8080/ws";

fn auth_header(token: impl fmt::Display) -> impl TryIntoHeaderPair {
    ("Authorization", format!("Bearer {token}"))
}

fn request_post(pathname: impl fmt::Display) -> ClientRequest {
    Client::new().post(format!("{API_URL}/{pathname}"))
}

fn request_ws_as(token: String, project_id: impl fmt::Display) -> ws::WebsocketsRequest {
    Client::new()
        .ws(format!("{WS_URL}/{project_id}"))
        .set_header("Sec-WebSocket-Protocol", format!("auth.{token}"))
}

async fn expect_send_json(send: SendClientRequest, status_code: StatusCode) -> Value {
    let mut response = send.await.unwrap();

    assert_eq!(response.status(), status_code);

    response.json().await.unwrap()
}

fn expect_json_value(value: &Value, key: &str) -> Value {
    value
        .get(key)
        .expect(&format!("{key:?} should exist"))
        .clone()
}

fn expect_json_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .expect(&format!("{key:?} should exist"))
        .as_str()
        .expect(&format!("{key:?} should be a string"))
        .to_string()
}

fn expect_json_object(value: &Value, key: &str) -> Map<String, Value> {
    value
        .get(key)
        .expect(&format!("{key:?} should exist in {value}"))
        .as_object()
        .expect(&format!("{key:?} should be an object in {value}"))
        .clone()
}

fn expect_ws_msg_json(msg: Option<Result<Frame, ProtocolError>>, expect_msg: &str) -> Value {
    if let Some(Ok(awc::ws::Frame::Text(txt))) = msg {
        serde_json::from_slice(&txt).unwrap()
    } else {
        panic!("{expect_msg}");
    }
}

async fn send_ws_msg<Ws>(ws: &mut Ws, msg: impl ToString)
where
    Ws: SinkExt<ws::Message> + Unpin,
    Ws::Error: fmt::Debug,
{
    ws.send(ws::Message::Text(msg.to_string().into()))
        .await
        .unwrap()
}

async fn login_as(guest_name: &str) -> (String, String) {
    let body = expect_send_json(
        request_post("auth/guest").send_json(&json!({ "guest_name": guest_name })),
        StatusCode::OK,
    )
    .await;

    (
        expect_json_string(&body, "jwt"),
        expect_json_string(&body, "id"),
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
    let project_id = expect_send_json(
        request_post("create/test_project")
            .insert_header(auth_header(&owner))
            .send(),
        StatusCode::CREATED,
    )
    .await;
    let project_id = expect_json_string(&project_id, "id");

    // --- 3. Connect both websockets ---
    let (_, mut owner_ws) = request_ws_as(owner, &project_id).connect().await.unwrap();

    // Owner handshake
    {
        let msg = expect_ws_msg_json(owner_ws.next().await, "Should receive welcome");
        let action = expect_json_value(&msg, "action");
        assert_eq!(action, "welcome");
        let users = expect_json_object(&msg, "users");
        assert_eq!(users.len(), 1, "Self should be included");
        assert!(users.contains_key(&owner_id), "Self should be included")
    }

    // Owner get notified about its own connection
    {
        let msg = expect_ws_msg_json(owner_ws.next().await, "Should receive own connection");
        let action = expect_json_string(&msg, "action");
        assert_eq!(action, "user_connected");
        let user_id = expect_json_string(&msg, "user_id");
        assert_eq!(user_id, owner_id);
    }

    let (_, mut guest_ws) = request_ws_as(guest, &project_id).connect().await.unwrap();

    // Guest handshake
    {
        let msg = expect_ws_msg_json(guest_ws.next().await, "Should receive welcome");
        let action = expect_json_value(&msg, "action");
        assert_eq!(action, "welcome");
        let users = expect_json_object(&msg, "users");
        assert_eq!(users.len(), 2, "Self should be included");
        assert!(users.contains_key(&guest_id), "Self should be included")
    }

    // Guest get notified about its own connection
    {
        let msg = expect_ws_msg_json(guest_ws.next().await, "Should receive own connection");
        let action = expect_json_string(&msg, "action");
        assert_eq!(action, "user_connected");
        let user_id = expect_json_string(&msg, "user_id");
        assert_eq!(user_id, guest_id);
    }

    // Owner get notified about new guest
    {
        let msg = expect_ws_msg_json(owner_ws.next().await, "Should receive guest connection");
        let action = expect_json_string(&msg, "action");
        assert_eq!(action, "user_connected");
        let user_id = expect_json_string(&msg, "user_id");
        assert_eq!(user_id, guest_id);
    }

    // --- 4. Gives editor to guest ---
    send_ws_msg(
        &mut owner_ws,
        json!({
            "action": "permit_access",
            "user_id": guest_id,
            "access": "editor",
        }),
    )
    .await;

    // Access update
    {
        let msg = expect_ws_msg_json(owner_ws.next().await, "Should receive access update");
        let action = expect_json_string(&msg, "action");
        assert_eq!(action, "update_access");
        let user_id = expect_json_string(&msg, "user_id");
        assert_eq!(user_id, guest_id);
        let access = expect_json_string(&msg, "access");
        assert_eq!(access, "editor");
    }

    {
        let msg = expect_ws_msg_json(guest_ws.next().await, "Should receive access update");
        let action = expect_json_string(&msg, "action");
        assert_eq!(action, "update_access");
        let user_id = expect_json_string(&msg, "user_id");
        assert_eq!(user_id, guest_id);
        let access = expect_json_string(&msg, "access");
        assert_eq!(access, "editor");
    }

    // --- 4. Inserts text in "test" file ---
    send_ws_msg(
        &mut guest_ws,
        json!({
            "action": "file_create",
            "file": "test",
        }),
    )
    .await;

    send_ws_msg(
        &mut guest_ws,
        json!({
            "action": "sync",
            "revision": 0,
            "file": "test",
            "actions": [{ "kind": "insertion", "from": 0, "text": "hello world", "owner": guest_id }],
        }),
    )
    .await;

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

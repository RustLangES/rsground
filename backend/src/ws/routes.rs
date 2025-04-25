use actix_error_proc::{proof_route, HttpResult};
use actix_web::{web, HttpRequest};
use uuid::Uuid;

use crate::auth::jwt;
use crate::http_errors::HttpErrors;
use crate::state::AppState;
use crate::ws::utils::parse_protocol_header;
use crate::ws::websocket::RgWebsocket;

fn get_element<'a>(key_val: &'a Vec<(String, String)>, target: &str) -> Option<&'a String> {
    key_val
        .iter()
        .find_map(|(key, val)| (key == target).then_some(val))
}

#[proof_route(get("/ws/{project_id}"))]
async fn websocket(
    data: web::Data<AppState>,
    project_id: web::Path<Uuid>,
    req: HttpRequest,
    stream: web::Payload,
) -> HttpResult<HttpErrors> {
    let (_, key_val) = parse_protocol_header(&req)?;

    let password = get_element(&key_val, "password").cloned();
    let Some(auth) = get_element(&key_val, "auth") else {
        return Err(HttpErrors::NoTokenProvided);
    };

    let user_info = jwt::decode(auth).ok_or(HttpErrors::InvalidJWT)?;

    let app_state = data.get_ref().clone();
    let ws = RgWebsocket::join_project(app_state, user_info, *project_id, password)?;

    let (response, session, stream) =
        actix_ws::handle(&req, stream).map_err(HttpErrors::WebsocketStart)?;

    let stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    ws.start(session, stream);

    Ok(response)
}

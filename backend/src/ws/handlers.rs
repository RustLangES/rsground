use std::sync::Arc;

use actix_error_proc::{proof_route, HttpResult};
use actix_web::{web, HttpRequest};

use crate::auth::jwt;
use crate::http_errors::HttpErrors;
use crate::models::project_access::ProjectAccess;
use crate::state::AppState;
use crate::ws::websocket::RgWebsocket;

#[proof_route(get("/ws"))]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<AppState>>,
) -> HttpResult<HttpErrors> {
    let user_info = jwt::get_user_info(&req)?;

    let ws = RgWebsocket {
        app_state: data.get_ref().clone(),
        user_info,
        access: ProjectAccess::None,
    };

    let (response, session, stream) =
        actix_ws::handle(&req, stream).map_err(HttpErrors::WebsocketStart)?;

    let stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    ws.start(session, stream);

    Ok(response)
}

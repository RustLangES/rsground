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
    data: web::Data<AppState>,
) -> HttpResult<HttpErrors> {
    let (protocols, key_val) = parse_protocol_header(&req)?;
    let Some(auth) = key_val.iter().find(|(auth, _)| auth == "auth") else {
        return Err(HttpErrors::NoTokenProvided);
    };
    let user_info = jwt::decode(&auth.1).ok_or(HttpErrors::InvalidJWT)?;

    let ws = RgWebsocket {
        app_state: data.get_ref().clone().into(),
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

fn parse_protocol_header(
    req: &HttpRequest,
) -> Result<(Vec<String>, Vec<(String, String)>), HttpErrors> {
    let mut key_value: Vec<(String, String)> = vec![];
    let mut protocols: Vec<String> = vec![];
    let Some(header) = req.headers().get("sec-websocket-protocol") else {
        return Err(HttpErrors::NoTokenProvided);
    };
    let header = header
        .to_str()
        .map_err(|_| return HttpErrors::NoTokenProvided)?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let elements = header.splitn(2, ",").into_iter().collect::<Vec<&str>>();

    for elem in elements {
        let value = elem.splitn(2, ".").collect::<Vec<&str>>();

        if value.len() > 1 {
            key_value.push((value[0].to_string(), value[1].to_string()));
        } else {
            protocols.push(value[0].to_string());
        }
    }

    return Ok((protocols, key_value));
}

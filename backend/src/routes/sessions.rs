use crate::{db, utils::{docker::{create_docker_session, run_container_code, set_container_file}, structures::{crates::is_toml_allowed, session::{SessionRetrievalError, SessionUpdate}, state::AppState}}};
use actix_web::{post, put, rt::spawn, web::{Data, Json, Payload, Query}, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::{handle, AggregatedMessage};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::to_string;
use sqlx::query;

macro_rules! respond_docker_operation {
    ($operation:expr, $result:expr) => {
        match $operation {
            Ok(res) => $result(res),
            Err(SessionRetrievalError::Error) => HttpResponse::InternalServerError().finish(),
            Err(SessionRetrievalError::NotFound) => HttpResponse::NotFound().finish()
        }
    };

    ($operation:expr) => {
        respond_docker_operation!($operation, |_| HttpResponse::Ok().finish())
    }
}

#[post("/session")]
pub async fn post_new_session() -> impl Responder {
    create_docker_session()
        .await
        .map_or(
            HttpResponse::InternalServerError()
                .into(),
            |body| HttpResponse::Ok()
                .content_type("text/utf8")
                .body(body)
        )
}

#[derive(Deserialize)]
pub struct SessionQuery {
    session: String
}

pub async fn get_session_ws(
    req: HttpRequest,
    query: Query<SessionQuery>,
    stream: Payload,
    app_state: Data<AppState>
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    match SessionUpdate::current_state(&query.session).await {
        Ok(current) => {
            session.text(
                to_string(&current)
                    .unwrap()
            )
                .await
                .unwrap();
        },

        Err(SessionRetrievalError::NotFound) => {
            return Ok(HttpResponse::BadRequest()
                .finish());
        },

        Err(SessionRetrievalError::Error) => {
            return Ok(HttpResponse::InternalServerError()
                .finish());
        }
    };

    let mut handler_session = session
        .clone();

    spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Ok(AggregatedMessage::Ping(ping)) = msg {
                handler_session.pong(&ping).await.unwrap();
            }
        }
    });

    let mut rx = app_state.session_update_tx.subscribe();

    spawn(async move {
        while let Ok(update) = rx.recv().await {
            session.text(to_string(&update).unwrap()).await.unwrap();
        }
    });

    Ok(res)
}

#[put("/session/name")]
pub async fn put_session_name(query: Query<SessionQuery>, name: String) -> impl Responder {
    query!(
        "UPDATE sessions SET name = ? WHERE hash = ?",
        name,
        query.session
    )
        .execute(db!())
        .await
        .map_or(
            HttpResponse::InternalServerError(),
            |result| match result.rows_affected() {
                0 => HttpResponse::NotFound(),
                _ => HttpResponse::NoContent()
            }
        )
}

#[put("/session/code")]
pub async fn put_session_code(query: Query<SessionQuery>, code: String) -> impl Responder {
    respond_docker_operation!{ set_container_file(&query.session, "/host/src/main.rs", &code).await }
}

#[put("/session/crates")]
pub async fn put_session_crates(query: Query<SessionQuery>, data: Json<SessionUpdate>) -> impl Responder {
    if !is_toml_allowed(data.crates.clone()) {
        return HttpResponse::BadRequest()
            .finish();
    }

    let crates_str = "
        [package]
        name=\"runner_project\"
        version=\"1.0.0\"
        edition=\"2021\"
    ".to_string() + &data.crates;

    respond_docker_operation!{ set_container_file(&query.session, "/host/Cargo.toml", &crates_str).await }
}

#[post("/session/run")]
pub async fn run_session_code(query: Query<SessionQuery>) -> impl Responder {
    respond_docker_operation!{
        run_container_code(&query.session).await,
        |res| HttpResponse::Ok()
            .content_type("text/utf8")
            .body(res)
    }
}

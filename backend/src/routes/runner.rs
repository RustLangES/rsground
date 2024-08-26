use actix_web::{post, put, rt::spawn, web::{Data, Json, Payload, Query}, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::{handle, AggregatedMessage};
use chrono::Utc;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::to_string;
use sqlx::query;
use crate::{db, utils::{docker::{create_docker_session, get_container_code, set_container_code}, structures::state::{AppState, SessionUpdate}}};

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

    let query = query!("SELECT * FROM sessions WHERE hash = ?", query.session)
        .fetch_one(db!())
        .await
        .ok()
        .take_if(|result| result
            .expires_at
            .lt(&Utc::now().naive_utc())
        );

    if let Some(query) = query {
        let Some(code) = get_container_code(query.hash).await
        else { return Ok(HttpResponse::InternalServerError().into()); };

        let update = to_string(&SessionUpdate {
            name: query.name,
            code
        }).unwrap();

        session.text(update).await.unwrap();
    } else {
        return Ok(HttpResponse::BadRequest().into());
    }

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

#[put("/session")]
pub async fn put_session_data(
    query: Query<SessionQuery>,
    data: Json<SessionUpdate>
) -> impl Responder {
    let updated = query!(
        "UPDATE sessions SET name = ? WHERE hash = ?",
        data.name,
        query.session
    )
        .execute(db!())
        .await
        .map(|result| result.rows_affected())
        .unwrap_or(0);

    if updated == 0 {
        return HttpResponse::BadRequest();
    }

    set_container_code(query.session.clone(), data.code.clone())
        .await
        .map_or(
            HttpResponse::InternalServerError(),
            |_| HttpResponse::Ok()
        )
}

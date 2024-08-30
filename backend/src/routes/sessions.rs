use std::ops::Add;
use crate::{db, utils::{docker::{abstractions::set_container_file, modules::{create_docker_session, run_container_code}}, structures::crates::is_toml_allowed}};
use actix_web::{post, web::{Json, Query}, HttpResponse, Responder};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::query;

#[derive(Deserialize)]
pub struct RegisterSessionQuery {
    session: Option<String>
}

#[post("/session")]
pub async fn post_new_session(query: Query<RegisterSessionQuery>) -> impl Responder {
    if query.session.is_none() {
        return create_docker_session()
            .await
            .map_or(
                HttpResponse::InternalServerError()
                    .into(),
                |body| HttpResponse::Ok()
                    .content_type("text/utf8")
                    .body(body)
            );
    }

    let session = query.session.as_ref().unwrap();
    let new_expiration = Utc::now().add(Duration::seconds(60));

    let update_result = query!(
        "UPDATE runners SET expires_at = ? WHERE hash = ?",
            new_expiration,
            session
    )
        .execute(db!())
        .await;

    match update_result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok(),
        Ok(_) => HttpResponse::NotFound(),
        Err(_) => HttpResponse::InternalServerError()
    }.into()
}

#[derive(Deserialize)]
pub struct SessionQuery {
    session: String,
    gist: Option<String>
}

#[derive(Deserialize)]
pub struct CodeToRun {
    crates: Option<String>,
    code: Option<String>
}

#[post("/session/run")]
pub async fn run_session_code(query: Query<SessionQuery>, program: Json<CodeToRun>) -> impl Responder {
    if let Some(crates) = &program.crates {
        if !is_toml_allowed(crates) {
            return HttpResponse::BadRequest()
                .into();
        }

        let crates = format!(
            "[package]\nname = \"runner_{}\"\nversion = \"0.0.1\"\n\n{}",
            &query.session,
            &crates
        );

        if let Err(err) = set_container_file(
            &query.session,
            "/host/Cargo.toml",
            &crates
        ).await {
            return err.to_http_response();
        };
    }

    if let Some(code) = &program.code {
        if let Err(err) = set_container_file(
            &query.session,
            "/host/src/main.rs",
            code
        ).await {
            return err.to_http_response();
        }
    } 

    match run_container_code(&query.session).await {
        Ok(res) => {
            if let Some(gist) = &query.gist {
                let expire_time = Utc::now().add(Duration::days(10));

                let query_is_error = query!(
                    "
                        INSERT INTO states(gist_id, last_run, expires_at)
                        VALUES ($1, $2, $3) ON CONFLICT(gist_id) DO UPDATE
                        SET last_run = $2
                    ",
                    gist,
                    res,
                    expire_time
                )
                    .execute(db!())
                    .await
                    .is_err();

                if query_is_error {
                    return HttpResponse::InternalServerError()
                        .finish();
                }
            }

            HttpResponse::Ok()
                .content_type("text/utf8")
                .body(res)
        },
        Err(err) => err.to_http_response()
    }
}

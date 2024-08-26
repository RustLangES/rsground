use actix_web::{get, web::Query, HttpResponse, Responder};
use serde::Deserialize;
use crate::utils::crates::fetch_crates_list;

#[derive(Deserialize, Clone)]
struct GetCratesQuery {
    query: Option<String>,
    page: Option<u32>
}

#[get("/crates")]
pub async fn get_crates_list(query: Query<GetCratesQuery>) -> impl Responder {
    fetch_crates_list(query.query.clone(), query.page)
        .await
        .and_then(|res| serde_json::to_string(&res).ok())
        .map_or(
            HttpResponse::InternalServerError()
                .into(),
            |res| HttpResponse::Ok()
                .content_type("application/json")
                .body(res)
        )
}

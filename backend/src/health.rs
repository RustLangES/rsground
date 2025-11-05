use actix_web::{HttpResponse, get};

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

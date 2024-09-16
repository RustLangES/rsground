use actix_web::{get, web::Query, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::utils::auth::{get_auth_link, may_authenticate};

#[get("/auth")]
pub async fn auth(req: HttpRequest) -> impl Responder {
    if !may_authenticate() {
        println!(
            "ERROR: This instance may not authenticate,
the GITHUB_APP_TOKEN and GITHUB_APP_ID
environment variables are missing"
        );

        return HttpResponse::BadRequest()
            .content_type("text/utf8")
            .body("This instance may not authenticate, check logs for more information.");
    } 

    let cinfo = req.connection_info();

    let base_url = format!(
        "{}://{}",
        cinfo.scheme(),
        cinfo.host()
    );

    get_auth_link(base_url + "/api/auth/callback")
        .map(|link| HttpResponse::PermanentRedirect()
            .insert_header(("Location", link))
            .finish()
        )
        .unwrap_or(
            HttpResponse::BadRequest()
                .finish()
        )
}

#[derive(Deserialize)]
struct GithubCallback {
    code: String
}

#[get("/auth/callback")]
pub async fn auth_callback(_req: HttpRequest, query: Query<GithubCallback>) -> impl Responder {
    println!("{}", query.code);
    HttpResponse::Ok()
}

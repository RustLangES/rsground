use actix_error_proc::{proof_route, HttpResult};
use actix_web::{get, web, HttpResponse, Responder};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::auth::{github, jwt};
use crate::http_errors::HttpErrors;

pub struct OAuthData {
    pub client: oauth2::basic::BasicClient,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[get("/auth")]
pub async fn oauth(oauth: web::Data<OAuthData>) -> impl Responder {
    let (auth_url, _csrf_token) = oauth
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/auth/callback")]
async fn auth_callback(
    query: web::Query<AuthRequest>,
    oauth_data: web::Data<OAuthData>,
) -> HttpResult<HttpErrors> {
    let code = AuthorizationCode::new(query.code.clone());

    let token = oauth_data
        .client
        .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|err| err.to_string())
        .map_err(HttpErrors::CodeExchange)
        .inspect_err(|err| log::error!("{err}"))?;

    let access_token = token.access_token().secret();
    let github_user = github::fetch_user(access_token)
        .await
        .map_err(HttpErrors::GithubUserFetch)
        .inspect_err(|err| log::error!("{err}"))?;

    let jwt =
        RgUserData::new(github_user.login.clone(), github_user.login.clone(), false).encode()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jwt": jwt,
        "id": github_user.login,
        "name": github_user.login,
        "avatar_url": github_user.avatar_url,
        "is_guest": false,
    })))
}

#[derive(Deserialize)]
struct GuestLoginRequest {
    guest_name: String,
}

#[proof_route(post("/login-guest"))]
async fn guest_jwt(body: web::Json<GuestLoginRequest>) -> HttpResult<HttpErrors> {
    let guest_name = &body.guest_name;
    let guest_uuid = Uuid::new_v4().to_string();
    let jwt = RgUserData::new(guest_uuid.clone(), guest_name.clone(), true).encode()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jwt": jwt,
        "id": guest_uuid,
        "name": guest_name,
        "is_guest": true,
    })))
}
#[derive(Deserialize)]
struct UpdateNameRequest {
    new_name: String,
}

#[proof_route(post("/update-name"))]
async fn update_name(
    body: web::Json<UpdateNameRequest>,
    req: actix_web::HttpRequest,
) -> HttpResult<HttpErrors> {
    let token_data = jwt::get_user_info(&req)?;

    if !token_data.is_guest {
        return Err(HttpErrors::GithubNameChange);
    }

    let uuid = token_data.id;
    let new_name = body.into_inner().new_name;

    let jwt = RgUserData::new(uuid.clone(), new_name.clone(), true).encode()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jwt": jwt,
        "id": uuid,
        "name": new_name,
    })))
}

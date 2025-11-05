use actix_failwrap::proof_route;
use actix_web::{HttpRequest, HttpResponse, get, web};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::auth::{github, jwt};
use crate::http_errors::HttpErrors;
use crate::state::AppState;
use crate::utils::ArcStr;

use super::auth_log;

pub struct OAuthData {
    pub client: oauth2::basic::BasicClient,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[get("/auth")]
pub async fn auth(oauth: web::Data<OAuthData>) -> HttpResponse {
    let (auth_url, _csrf_token) = oauth
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[proof_route("GET /auth/me")]
pub async fn me(req: HttpRequest) -> Result<HttpResponse, HttpErrors> {
    let user_info = jwt::get_user_info(&req)?;

    Ok(HttpResponse::Ok().json(user_info))
}

#[proof_route("GET /auth/callback")]
async fn callback(
    state: web::Data<AppState>,
    query: web::Query<AuthRequest>,
    oauth_data: web::Data<OAuthData>,
) -> Result<HttpResponse, HttpErrors> {
    let code = AuthorizationCode::new(query.code.clone());

    let token = oauth_data
        .client
        .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|err| err.to_string())
        .map_err(HttpErrors::CodeExchange)
        .inspect_err(|err| auth_log::callback::error!("{err}"))?;

    let access_token = token.access_token().secret();
    let github_user = github::fetch_user(access_token)
        .await
        .map_err(HttpErrors::GithubUserFetch)
        .inspect_err(|err| auth_log::callback::error!("{err}"))?;

    let user_data = RgUserData::new(
        github_user.login.as_str().into(),
        github_user.login.as_str().into(),
        false,
    );

    state
        .add_username(user_data.id.clone(), user_data.name.clone())
        .await;

    let jwt = user_data.encode()?;

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

#[proof_route("POST /auth/guest")]
async fn login_guest(
    state: web::Data<AppState>,
    body: web::Json<GuestLoginRequest>,
) -> Result<HttpResponse, HttpErrors> {
    let guest_name = &body.guest_name;
    let guest_uuid = Uuid::new_v4().to_string();

    let user_data = RgUserData::new(
        guest_uuid.as_str().into(),
        guest_name.as_str().into(),
        false,
    );

    state
        .add_username(user_data.id.clone(), user_data.name.clone())
        .await;

    let jwt = user_data.encode()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jwt": jwt,
        "id": guest_uuid,
        "name": guest_name,
        "is_guest": true,
    })))
}
#[derive(Deserialize)]
struct UpdateNameRequest {
    new_name: ArcStr,
}

#[proof_route("POST /auth/update")]
async fn update_name(
    state: web::Data<AppState>,
    body: web::Json<UpdateNameRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, HttpErrors> {
    let token_data = jwt::get_user_info(&req)?;

    if !token_data.is_guest {
        return Err(HttpErrors::GithubNameChange);
    }

    let uuid = token_data.id;
    let new_name = body.into_inner().new_name;

    let user_data = RgUserData::new(uuid.clone(), new_name.clone(), false);

    state
        .add_username(user_data.id.clone(), user_data.name.clone())
        .await;

    let jwt = user_data.encode()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jwt": jwt,
        "id": uuid,
        "name": new_name,
    })))
}

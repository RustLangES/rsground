use std::sync::LazyLock;

use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::expect_var;
use crate::http_errors::HttpErrors;

pub static JWT_SECRET: LazyLock<String> = LazyLock::new(|| expect_var!("JWT_SECRET"));

#[derive(Debug, Serialize, Deserialize)]
pub struct RgUserData {
    pub id: String,
    pub name: String,
    pub is_guest: bool,
    pub exp: i64,
}

impl RgUserData {
    pub fn new(id: String, name: String, is_guest: bool) -> Self {
        let exp = Utc::now() + Duration::hours(12);

        Self {
            id,
            name,
            is_guest,
            exp: exp.timestamp(),
        }
    }

    pub fn encode(self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(self)
    }
}

pub fn encode(data: RgUserData) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode::<RgUserData>(
        &jsonwebtoken::Header::default(),
        &data,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .inspect_err(|err| log::error!("Error encoding jwt: {err}"))
}

pub fn decode(token: impl AsRef<str>) -> Option<RgUserData> {
    let token_data = jsonwebtoken::decode::<RgUserData>(
        token.as_ref(),
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .inspect_err(|err| {
        log::error!("Error al decodificar JWT: {err}");
    })
    .ok()?;

    if Utc::now().timestamp() >= token_data.claims.exp {
        return None;
    }

    Some(token_data.claims)
}

pub fn get_auth_token(req: &HttpRequest) -> Option<&str> {
    req.headers()
        .get("Authorization")
        .and_then(|auth| auth.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
}

pub fn get_user_info(req: &HttpRequest) -> Result<RgUserData, HttpErrors> {
    let token = get_auth_token(&req).ok_or_else(|| HttpErrors::NoTokenProvided)?;
    decode(token).ok_or(HttpErrors::InvalidJWT)
}

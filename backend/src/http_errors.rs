use actix_failwrap::ErrorResponse;
use thiserror::Error;

#[derive(ErrorResponse, Error, Debug)]
pub enum HttpErrors {
    // -- JWT/Auth related -- //
    #[error("Error encoding JWT: {0}")]
    EncodingJWT(#[from] jsonwebtoken::errors::Error),

    /// oauth2 use long types, so we use a pre-rendered string
    #[error("Error in code exchange: {0}")]
    CodeExchange(String),

    #[error("Only guest users can change their name")]
    #[status_code(Forbidden)]
    GithubNameChange,

    #[error("Error fetching github user: {0}")]
    GithubUserFetch(reqwest::Error),

    #[error("Invalid token")]
    #[status_code(Unauthorized)]
    InvalidJWT,

    #[error("No token provided")]
    #[status_code(Unauthorized)]
    NoTokenProvided,

    // -- Websockets related -- //
    #[error("Project doesn't exist")]
    #[status_code(NotFound)]
    ProjectDoesNotExist,

    #[error("Do not have access to project")]
    #[status_code(Unauthorized)]
    NotAccessible,

    #[error("Invalid password for private project")]
    #[status_code(Unauthorized)]
    InvalidPassword,

    #[error("Error at websocket start: {0}")]
    WebsocketStart(actix_web::Error),
}

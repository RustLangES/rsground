use actix_error_proc::ActixError;
use thiserror::Error;

#[derive(ActixError, Error, Debug)]
pub enum HttpErrors {
    // -- JWT/Auth related -- //
    #[error("Error encoding JWT: {0}")]
    EncodingJWT(#[from] jsonwebtoken::errors::Error),

    /// oauth2 use long types, so we use a pre-rendered string
    #[error("Error in code exchange: {0}")]
    CodeExchange(String),

    #[error("Only guest users can change their name")]
    #[http_status(Forbidden)]
    GithubNameChange,

    #[error("Error fetching github user: {0}")]
    GithubUserFetch(reqwest::Error),

    #[error("Invalid token")]
    #[http_status(Unauthorized)]
    InvalidJWT,

    #[error("No token provided")]
    #[http_status(Unauthorized)]
    NoTokenProvided,

    // -- Websockets related -- //
    #[error("Project doesn't exist")]
    #[http_status(NotFound)]
    ProjectDoesNotExist,

    #[error("Do not have access to project")]
    #[http_status(Unauthorized)]
    NotAccessible,

    #[error("Invalid password for private project")]
    #[http_status(Unauthorized)]
    InvalidPassword,

    #[error("Error at websocket start: {0}")]
    WebsocketStart(actix_web::Error),
}

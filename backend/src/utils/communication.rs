use std::fmt::{Display, Formatter, Result as FmtResult, Error as FmtError};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use thiserror::Error;
use serde_repr::*;

#[derive(Debug, Error, Serialize)]
pub enum ActError {
    #[error("Undisclosed error while acting on behalf this request")]
    InternalServerError,

    #[error("The submited TOML goes beyond restrictions")]
    InvalidToml,

    #[error("This request requires content")]
    MissingContent
}

pub trait RequestActor {
    type ContentType;

    fn act(&self, op: &RunnerRequestOp, content: &Option<Self::ContentType>)
    -> Result<(), ActError>;
}

#[derive(Deserialize_repr)]
#[repr(u8)]
pub enum RunnerRequestOp {
    RunCode = 1 << 0,
    UploadCode = 1 << 1,
    UpdateCargo = 1 << 2,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum RunnerResponseOp {
    Aknowledge = 1 << 0,
    InternalServerError = 1 << 1,
    ParseError = 1 << 2,
    RustAnalyzer = 1 << 3,
    InvalidToml = 1 << 4,
    MissingContent = 1 << 5
}

#[derive(Deserialize)]
pub struct RunnerRequest<TMessage> {
    op: RunnerRequestOp,
    content: Option<TMessage>
}

#[derive(Serialize)]
pub struct RunnerResponse<TMessage> {
    op: RunnerResponseOp,
    content: Option<TMessage>
}

impl<TMessage> RunnerRequest<TMessage> {
    pub fn op_code(&self) -> &RunnerRequestOp {
        &self.op
    }

    pub fn content(&self) -> &Option<TMessage> {
        &self.content
    }

    pub fn act(&self, actor: &impl RequestActor<ContentType = TMessage>) -> RunnerResponse<ActError> {
        match actor.act(self.op_code(), self.content()) {
            Ok(_) => RunnerResponse::aknowledge(),
            Err(err) => RunnerResponse::from(err)
        }
    }
}

impl<'r, TMessage: Deserialize<'r>> TryFrom<&'r String> for RunnerRequest<TMessage> {
    type Error = RunnerResponse<()>;

    fn try_from(value: &'r String) -> Result<Self, Self::Error> {
        from_str::<Self>(value).map_err(|_| RunnerResponse::parse_error() )
    }
}

impl<TMessage> RunnerResponse<TMessage> {
    pub fn new(op: RunnerResponseOp, content: Option<TMessage>) -> Self {
        Self { op, content }
    }

    pub fn aknowledge() -> Self {
        Self::new(RunnerResponseOp::Aknowledge, None)
    }

    pub fn internal_server_error(error: TMessage) -> Self {
        Self::new(RunnerResponseOp::InternalServerError, Some(error))
    }

    pub fn parse_error() -> Self {
        Self::new(RunnerResponseOp::ParseError, None)
    }

    pub fn invalid_toml(error: TMessage) -> Self {
        Self::new(RunnerResponseOp::InvalidToml, Some(error))
    }

    pub fn missing_content(error: TMessage) -> Self {
        Self::new(RunnerResponseOp::MissingContent, Some(error))
    }
}

impl From<ActError> for RunnerResponse<ActError> {
    fn from(value: ActError) -> Self {
        match value {
            ActError::InternalServerError => Self::internal_server_error(value),
            ActError::InvalidToml => Self::invalid_toml(value),
            ActError::MissingContent => Self::missing_content(value)
        }
    }
}

impl<TMessage: Serialize> Display for RunnerResponse<TMessage> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", to_string(self).map_err(|_| FmtError)?)
    }
}

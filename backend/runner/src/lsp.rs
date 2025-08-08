use core::fmt;

use lsp_types::notification::Notification;
use lsp_types::request::Request;
use serde::Deserialize;

pub use lsp_types::*;

const JSONRPC: &str = "2.0";

pub struct LspInput;

impl LspInput {
    pub fn notify<T: Notification>(params: T::Params) -> Result<String, serde_json::Error> {
        serde_json::to_string(&serde_json::json!({
            "jsonrpc": JSONRPC,
            "method": T::METHOD,
            "params": serde_json::to_value(params)?
        }))
    }

    pub fn request<T: Request>(
        id: impl serde::Serialize,
        params: T::Params,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string(&serde_json::json!({
            "jsonrpc": JSONRPC,
            "id": id,
            "method": T::METHOD,
            "params": serde_json::to_value(params)?
        }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LspId {
    String(String),
    Number(u32),
}

impl fmt::Display for LspId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspId::String(s) => fmt::Display::fmt(s, f),
            LspId::Number(n) => fmt::Display::fmt(n, f),
        }
    }
}

impl PartialEq<u32> for &LspId {
    fn eq(&self, other: &u32) -> bool {
        match self {
            LspId::String(_) => false,
            LspId::Number(this) => this == other,
        }
    }
}

impl PartialEq<u32> for LspId {
    fn eq(&self, other: &u32) -> bool {
        match self {
            LspId::String(_) => false,
            LspId::Number(this) => this == other,
        }
    }
}

impl PartialEq<str> for LspId {
    fn eq(&self, other: &str) -> bool {
        match self {
            LspId::String(this) => this == other,
            LspId::Number(_) => false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LspResponse {
    Ok {
        jsonrpc: String,
        id: LspId,
        result: serde_json::Value,
    },
    Err {
        jsonrpc: String,
        id: LspId,
        error: serde_json::Value,
    },
}

impl LspResponse {
    pub fn id(&self) -> &LspId {
        match &self {
            LspResponse::Ok { id, .. } => id,
            LspResponse::Err { id, .. } => id,
        }
    }

    /// Returns `true` if the lsp response is [`Ok`].
    ///
    /// [`Ok`]: LspResponse::Ok
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok { .. })
    }

    /// Returns `true` if the lsp response is [`Err`].
    ///
    /// [`Err`]: LspResponse::Err
    #[must_use]
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err { .. })
    }
}

#[derive(Debug, Deserialize)]
pub struct LspNotify {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct LspRequest {
    pub jsonrpc: String,
    pub id: LspId,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LspOutput {
    Response(LspResponse),
    Request(LspRequest),
    Notify(LspNotify),
}

impl LspOutput {
    pub fn id(&self) -> Option<&LspId> {
        match self {
            LspOutput::Response(lsp_response) => Some(lsp_response.id()),
            LspOutput::Request(lsp_request) => Some(&lsp_request.id),
            LspOutput::Notify(_) => None,
        }
    }

    pub fn method(&self) -> Option<&String> {
        match self {
            LspOutput::Response(_) => None,
            LspOutput::Request(lsp_request) => Some(&lsp_request.method),
            LspOutput::Notify(lsp_notify) => Some(&lsp_notify.method),
        }
    }

    pub fn params(&self) -> Option<&serde_json::Value> {
        match self {
            LspOutput::Response(_) => None,
            LspOutput::Request(lsp_request) => Some(&lsp_request.params),
            LspOutput::Notify(lsp_notify) => Some(&lsp_notify.params),
        }
    }

    /// Returns `true` if the lsp output is [`Request`].
    ///
    /// [`Request`]: LspOutput::Request
    #[must_use]
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Request(..))
    }

    pub fn as_request(&self) -> Option<&LspRequest> {
        if let Self::Request(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the lsp output is [`Notify`].
    ///
    /// [`Notify`]: LspOutput::Notify
    #[must_use]
    pub fn is_notify(&self) -> bool {
        matches!(self, Self::Notify(..))
    }

    pub fn as_notify(&self) -> Option<&LspNotify> {
        if let Self::Notify(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the lsp output is [`Response`].
    ///
    /// [`Response`]: LspOutput::Response
    #[must_use]
    pub fn is_response(&self) -> bool {
        matches!(self, Self::Response(..))
    }

    pub fn as_response(&self) -> Option<&LspResponse> {
        if let Self::Response(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

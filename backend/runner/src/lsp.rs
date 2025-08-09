use core::fmt;

use lsp_types::notification::Notification;
use lsp_types::request::Request;
use serde::{Deserialize, Serialize};

pub use lsp_types::*;

pub struct LspInput;

impl LspInput {
    pub fn notify<T: Notification>(params: T::Params) -> Result<String, serde_json::Error> {
        serde_json::to_string(&serde_json::json!({
            "jsonrpc": JsonRpcVersion,
            "method": T::METHOD,
            "params": serde_json::to_value(params)?
        }))
    }

    pub fn request<T: Request>(
        id: impl serde::Serialize,
        params: T::Params,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string(&serde_json::json!({
            "jsonrpc": JsonRpcVersion,
            "id": id,
            "method": T::METHOD,
            "params": serde_json::to_value(params)?
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LspId {
    String(String),
    Number(u32),
}

impl LspId {
    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LspResponse {
    Ok {
        jsonrpc: JsonRpcVersion,
        id: LspId,
        result: serde_json::Value,
    },
    Err {
        jsonrpc: JsonRpcVersion,
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
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct LspRequest {
    pub jsonrpc: JsonRpcVersion,
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

// see: https://github.com/pr2502/ra-multiplex/blob/5bf0cf71de5853092ae4fbd7a859837f122f0f1b/src/lsp/jsonrpc.rs#L100-L138
/// ZST representation of the `"2.0"` version string
#[derive(Clone, Copy, Debug)]
pub struct JsonRpcVersion;

impl serde::ser::Serialize for JsonRpcVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str("2.0")
    }
}

impl<'de> serde::de::Visitor<'de> for JsonRpcVersion {
    type Value = JsonRpcVersion;

    fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(r#"string value "2.0""#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "2.0" => Ok(JsonRpcVersion),
            _ => Err(E::custom("unsupported JSON-RPC version")),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(JsonRpcVersion)
    }
}

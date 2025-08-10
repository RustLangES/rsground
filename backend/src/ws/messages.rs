use core::fmt;
use std::collections::HashMap;
use std::sync::Arc;

use operational_transform::OperationSeq;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::collab::{Document, DocumentInfo, UserOperation};
use crate::project::AccessLevel;
use crate::utils::ArcStr;

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientMessage {
    Config {
        name: Option<ArcStr>,
        is_public: Option<bool>,
        password: Option<String>,
    },
    Execute,
    FileCreate {
        file: ArcStr,
    },
    FileDelete {
        file: ArcStr,
    },
    PermitAccess {
        user_id: ArcStr,
        access: AccessLevel,
    },
    StopExecute,
    Sync {
        file: ArcStr,
        revision: usize,
        actions: OperationSeq,
    },
    SyncCursor {
        file: ArcStr,
        cursors: Vec<(u32, u32)>,
    },
    SyncFiles,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerMessage {
    Error {
        message: String,
    },
    ProjectConfig {
        name: ArcStr,
        is_public: bool,
        password: Option<String>,
    },
    ProjectFiles {
        /// List of all file paths
        files: HashMap<ArcStr, DocumentInfo>,
    },
    UpdateAccess {
        access: AccessLevel,
        user_id: ArcStr,
    },
    UserConnected {
        user_id: ArcStr,
        user_name: ArcStr,
    },
    RequestAccess {
        user_id: ArcStr,
        user_name: ArcStr,
    },
    Sync {
        file: ArcStr,
        revision: usize,
        actions: Vec<UserOperation>,
    },
    SyncCursors {
        file: ArcStr,
        cursors: HashMap<ArcStr, Vec<(u32, u32)>>,
    },
    SyncOutput {
        channel: OutputChannel,
        buf: Vec<u8>,
    },
    SyncOutputStart,
    SyncOutputEnd {
        exit_code: u8,
    },
    Welcome {
        session_id: ArcStr,
        files: HashMap<ArcStr, DocumentInfo>,
        users: HashMap<ArcStr, (ArcStr, AccessLevel)>,
        // Only for owner
        requests: Option<HashMap<ArcStr, ArcStr>>,
    },
}

impl fmt::Display for ServerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMessage::Error { message } => f.debug_tuple("Error").field(message).finish(),
            ServerMessage::ProjectConfig {
                name, is_public, ..
            } => f
                .debug_tuple("ProjectConfig")
                .field(name)
                .field(is_public)
                .finish(),
            ServerMessage::ProjectFiles { .. } => f.debug_tuple("ProjectFiles").finish(),
            ServerMessage::UpdateAccess { access, user_id } => f
                .debug_tuple("UpdateAccess")
                .field(user_id)
                .field(access)
                .finish(),
            ServerMessage::UserConnected { user_id, user_name } => f
                .debug_tuple("UserConnected")
                .field(user_id)
                .field(user_name)
                .finish(),
            ServerMessage::RequestAccess { user_id, user_name } => f
                .debug_tuple("RequestAccess")
                .field(user_id)
                .field(user_name)
                .finish(),
            ServerMessage::Sync { file, revision, .. } => {
                f.debug_tuple("Sync").field(file).field(revision).finish()
            }
            ServerMessage::SyncCursors { file, .. } => {
                f.debug_tuple("SyncCursors").field(file).finish()
            }
            ServerMessage::SyncOutput { channel, .. } => {
                f.debug_tuple("SyncOutput").field(channel).finish()
            }
            ServerMessage::SyncOutputStart => f.debug_tuple("SyncOutputStart").finish(),
            ServerMessage::SyncOutputEnd { exit_code } => {
                f.debug_tuple("SyncOutputEnd").field(exit_code).finish()
            }
            ServerMessage::Welcome { session_id, .. } => {
                f.debug_tuple("Welcome").field(session_id).finish()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputChannel {
    Stdout,
    Stderr,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerMessageError {
    #[error("")]
    None,

    #[error("File {0:?} not found")]
    FileNotFound(ArcStr),

    #[error("You don't have read permission")]
    NotAccessible,

    #[error("You are not the owner: fork the project")]
    NotOwner,

    #[error("Project {0:?} not found")]
    ProjectNotFound(Uuid),

    #[error("Denied permission: read-only access")]
    ReadonlyPermission,
}

impl From<ServerMessageError> for ServerMessage {
    fn from(value: ServerMessageError) -> ServerMessage {
        ServerMessage::Error {
            message: value.to_string(),
        }
    }
}

#[derive(Clone)]
pub enum InternalMessage {
    FileEdit { path: ArcStr },
    FileCreate { path: ArcStr, doc: Arc<Document> },
    FileDelete { path: ArcStr },
}

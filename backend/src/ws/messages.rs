use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::collab::{Action, DocumentInfo};
use crate::project::AccessLevel;

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientMessage {
    PermitAccess {
        user_id: String,
        access: AccessLevel,
    },
    FileCreate {
        file: String,
    },
    FileDelete {
        file: String,
    },
    Sync {
        file: String,
        revision: usize,
        actions: Vec<Action>,
    },
    SyncCursor {
        file: String,
        cursors: Vec<(usize, usize)>,
    },
    SyncFiles,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerMessage {
    Error {
        message: String,
    },
    ProjectFiles {
        /// List of all file paths
        files: HashMap<String, DocumentInfo>,
    },
    UpdateAccess {
        access: AccessLevel,
        user_id: String,
    },
    UserConnected {
        user_id: String,
    },
    Sync {
        file: String,
        revision: usize,
        actions: Vec<Action>,
    },
    SyncCursors {
        file: String,
        cursors: HashMap<String, Vec<(usize, usize)>>,
    },
    Welcome {
        session_id: String,
        files: HashMap<String, DocumentInfo>,
        users: HashMap<String, AccessLevel>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ServerMessageError {
    #[error("")]
    None,

    #[error("File {0:?} not found")]
    FileNotFound(String),

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

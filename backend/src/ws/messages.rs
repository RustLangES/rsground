use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::document::Action;
use crate::models::project_access::AccessLevel;

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientMessage {
    CreateProject {
        name: String,
    },
    Delete {
        file: String,
        range_start: usize,
        range_end: usize,
    },
    ForkProject {
        project_id: Uuid,
    },
    GetProjectFiles,
    Insert {
        file: String,
        pos: usize,
        text: String,
    },
    JoinProject {
        project_id: Uuid,
        password: Option<String>,
    },
    PermitAccess {
        user_id: String,
        access: AccessLevel,
    },
    Sync {
        file: String,
        last_timestamp: u64,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerMessage {
    Error {
        message: String,
    },
    /// Trigger to someone join the project
    JoinedProject {
        access: AccessLevel,
        user_id: String,
    },
    ProjectCreated {
        project_id: Uuid,
    },
    ProjectFiles {
        /// List of all file paths
        files: Vec<String>,
    },
    ProjectForked {
        project_id: Uuid,
    },
    Update {
        file: String,
        content: String,
    },
    UpdateAccess {
        access: AccessLevel,
        user_id: String,
    },
    UserConnected {
        user_id: String,
    },
    SyncActions {
        file: String,
        actions: Vec<Action>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ServerMessageError {
    #[error("")]
    None,

    #[error("File {0:?} not found")]
    FileNotFound(String),

    #[error("Invalid password for private project")]
    InvalidPassword,

    #[error("You don't have read permission")]
    NotAccessible,

    #[error("You are not in a project")]
    NotInProject,

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

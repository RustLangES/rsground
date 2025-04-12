use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::document::Action;
use crate::models::file_node::FileNode;
use crate::models::project_access::AccessLevel;

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientMessage {
    CreateProject {
        name: String,
    },
    JoinProject {
        project_id: Uuid,
        password: Option<String>,
    },
    Insert {
        file: String,
        pos: usize,
        text: String,
    },
    Delete {
        file: String,
        range_start: usize,
        range_end: usize,
    },
    Sync {
        file: String,
        last_timestamp: u64,
    },
    GetProjectFiles,
    GrantEditor {
        user_id: String,
    },
    PermitAccess {
        username: String,
        access: AccessLevel,
    },
    ForkProject {
        project_id: Uuid,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerMessage {
    UserConnected {
        user_id: String,
    },
    ProjectCreated {
        project_id: Uuid,
    },
    SyncActions {
        file: String,
        actions: Vec<Action>,
    },
    Error {
        message: String,
    },
    ProjectFiles {
        /// List of all file paths
        files: Vec<String>,
    },
    JoinedProject {
        project_id: Uuid,
    },
    EditorRequestReceived {
        project_id: Uuid,
    },
    NewEditorRequest {
        project_id: Uuid,
        user_id: String,
    },
    EditorGranted {
        project_id: Uuid,
    },
    Update {
        project_id: Uuid,
        file: String,
        content: String,
    },
    ProjectForked {
        project_id: Uuid,
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

    #[error("User doesn't ask for editor-mode")]
    NobodyAskYou,

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

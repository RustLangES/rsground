use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ws::messages::ServerMessageError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    ReadOnly,
    Editor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectAccess {
    None,
    ReadOnly(Uuid),
    Editor(Uuid),
}

impl ProjectAccess {
    pub fn get_project_id(&self) -> Option<&Uuid> {
        match self {
            ProjectAccess::None => None,
            ProjectAccess::ReadOnly(ref uuid) => Some(uuid),
            ProjectAccess::Editor(ref uuid) => Some(uuid),
        }
    }

    #[expect(dead_code)]
    pub fn to_level(&self) -> Option<AccessLevel> {
        match self {
            ProjectAccess::None => None,
            ProjectAccess::ReadOnly(_) => Some(AccessLevel::ReadOnly),
            ProjectAccess::Editor(_) => Some(AccessLevel::Editor),
        }
    }

    /// If has read permissions, readonly or editor mode
    pub fn is_reader(&self) -> bool {
        matches!(self, Self::ReadOnly(..) | Self::Editor(..))
    }

    #[expect(dead_code)]
    pub fn is_readonly(&self) -> bool {
        matches!(self, Self::ReadOnly(..))
    }

    #[expect(dead_code)]
    pub fn is_editor(&self) -> bool {
        matches!(self, Self::Editor(..))
    }

    pub fn need_editor(&self) -> Result<&Uuid, ServerMessageError> {
        match self {
            ProjectAccess::None => Err(ServerMessageError::NotInProject),
            ProjectAccess::ReadOnly(_) => Err(ServerMessageError::ReadonlyPermission),
            ProjectAccess::Editor(ref uuid) => Ok(uuid),
        }
    }
}

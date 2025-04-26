use serde::{Deserialize, Serialize};

use crate::ws::messages::ServerMessageError;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    #[default]
    Queue,
    ReadOnly,
    Editor,
}

impl AccessLevel {
    pub fn can_read(&self) -> bool {
        matches!(self, Self::ReadOnly | Self::Editor)
    }

    #[expect(dead_code)]
    pub fn is_readonly(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    #[expect(dead_code)]
    pub fn is_editor(&self) -> bool {
        matches!(self, Self::Editor)
    }

    pub fn need_read(&self) -> Result<(), ServerMessageError> {
        self.can_read()
            .then_some(())
            .ok_or(ServerMessageError::NotAccessible)
    }

    pub fn need_editor(&self) -> Result<(), ServerMessageError> {
        match self {
            Self::Queue => Err(ServerMessageError::NotAccessible),
            Self::ReadOnly => Err(ServerMessageError::ReadonlyPermission),
            Self::Editor => Ok(()),
        }
    }
}

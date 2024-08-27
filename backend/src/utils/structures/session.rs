use serde::{Deserialize, Serialize};
use sqlx::query;
use crate::{db, utils::docker::get_container_file};

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionUpdate {
    pub name: String,
    pub code: String,
    pub crates: String
}

pub enum SessionRetrievalError {
    Error,
    NotFound
}

impl SessionUpdate {
    pub async fn current_state(hash: impl ToString) -> Result<SessionUpdate, SessionRetrievalError> {
        let hash = hash.to_string();

        let name = query!("SELECT name FROM sessions WHERE hash = ? AND expires_at > date('now')", hash)
            .fetch_one(db!())
            .await
            .map_err(|_| SessionRetrievalError::NotFound)
            .map(|query| query.name)?;

        let code = get_container_file(&hash, "/host/src/main.rs")
            .await?;

        let mut crates = get_container_file(&hash, "/host/Cargo.toml")
            .await?;

        crates = crates.split_once("\n\n")
            .map(|lines| lines.1)
            .ok_or(SessionRetrievalError::Error)?
            .to_string();

        Ok(Self {
            name,
            code,
            crates
        })
    }
}

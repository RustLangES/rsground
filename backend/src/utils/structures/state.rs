use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

pub struct AppState {
    pub session_update_tx: Sender<SessionUpdate>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionUpdate {
    pub name: String,
    pub code: String
}

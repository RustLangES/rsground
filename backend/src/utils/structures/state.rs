use tokio::sync::broadcast::Sender;
use super::session::SessionUpdate;


pub struct AppState {
    pub session_update_tx: Sender<SessionUpdate>
}

use std::sync::{Arc, Mutex, MutexGuard};

use crate::models::project::ProjectManager;

#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<Mutex<ProjectManager>>,
}

impl AppState {
    /// Lock project manager, and recover from poison mutex
    pub fn get_manager(&self) -> MutexGuard<'_, ProjectManager> {
        self.manager.lock().unwrap_or_else(|e| e.into_inner())
    }
}

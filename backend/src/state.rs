use crate::models::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub(crate) manager: Arc<Mutex<ProjectManager>>,
}

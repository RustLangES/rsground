use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::collab::Document;
use crate::project::project_log;
use crate::utils::ArcStr;
use crate::ws::messages::ServerMessageError;

use super::Project;

const MAIN_RS: &str = r#"fn main() {
    for i in 0..30 {
        println!("[{i}] \x1b[31mHello \x1b[1mWorld\x1b[0m!");
        std::thread::sleep(std::time::Duration::from_millis(100))
    }
}"#;

pub struct ProjectManager {
    projects: HashMap<Uuid, Arc<RwLock<Project>>>,
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            projects: HashMap::new(),
        }
    }

    pub async fn new_project(&mut self, owner: &RgUserData, name: ArcStr) -> Arc<RwLock<Project>> {
        let mut project = Project::new(owner.id.clone(), name).await;

        project
            .add_file("main.rs", Document::new_with(MAIN_RS.to_string()))
            .await;

        self.add_project(project)
    }

    pub fn add_project(&mut self, project: Project) -> Arc<RwLock<Project>> {
        project_log::info!("New {}: {}", project.id, project.name);
        self.projects
            .entry(project.id)
            .insert_entry(RwLock::new(project).into())
            .get()
            .clone()
    }

    pub fn get_project(&self, id: Uuid) -> Result<Arc<RwLock<Project>>, ServerMessageError> {
        self.projects
            .get(&id)
            .cloned()
            .ok_or(ServerMessageError::ProjectNotFound(id))
    }
}

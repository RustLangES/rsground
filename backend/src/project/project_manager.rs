use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::collab::Document;
use crate::utils::ArcStr;
use crate::ws::messages::ServerMessageError;

use super::Project;

const MAIN_RS: &str = r#"fn main() {
  println!("Hello World");
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

    pub fn new_project(&mut self, owner: &RgUserData, name: ArcStr) -> Arc<RwLock<Project>> {
        let mut project = Project::new(owner.id.clone(), name);

        project.add_file("main.rs", Document::new_with(MAIN_RS.to_string()));

        self.add_project(project)
    }

    pub fn add_project(&mut self, project: Project) -> Arc<RwLock<Project>> {
        log::info!("New project {}: {}", project.id, project.name);
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

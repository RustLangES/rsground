use std::collections::HashMap;

use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::collab::Document;
use crate::ws::messages::ServerMessageError;

use super::Project;

const MAIN_RS: &str = r#"fn main() {
  println!("Hello World");
}"#;

pub struct ProjectManager {
    projects: HashMap<Uuid, Project>,
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            projects: HashMap::new(),
        }
    }

    pub fn new_project(&mut self, owner: &RgUserData, name: impl Into<String>) -> &mut Project {
        let mut project = Project::new(owner.id.clone(), name);

        project.add_file("main.rs", Document::new_with(MAIN_RS.to_string()));

        self.add_project(project)
    }

    pub fn add_project(&mut self, project: Project) -> &mut Project {
        let project_id = project.id.clone();

        log::info!("New project {}: {}", project.id, project.name);
        self.projects.insert(project_id.clone(), project);

        // SAFETY: the project is just inserted above
        unsafe { self.projects.get_mut(&project_id).unwrap_unchecked() }
    }

    pub fn get_project(&self, id: &Uuid) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn get_project_mut(&mut self, id: Uuid) -> Result<&mut Project, ServerMessageError> {
        self.projects
            .get_mut(&id)
            .ok_or_else(|| ServerMessageError::ProjectNotFound(id))
    }
}

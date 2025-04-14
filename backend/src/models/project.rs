use std::collections::HashMap;

use serde::Serialize;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::ws::ws_ext::SessionExt;

use super::document::{generate_unique_replica_id, Document};
use super::project_access::AccessLevel;

#[derive(Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub documents: HashMap<String, Document>,
    pub allowed_users: HashMap<String, AccessLevel>,
    pub pending_requests: HashMap<String, actix_ws::Session>,
    pub is_public: bool,
    pub password: Option<String>,
    // FIXME: Remove closed sessions
    pub sessions: Vec<actix_ws::Session>,
}

impl Project {
    pub fn new(owner: String, name: impl Into<String>) -> Self {
        Project {
            id: Uuid::new_v4(),
            name: name.into(),
            owner,
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            pending_requests: HashMap::new(),
            is_public: true,
            password: None,
            sessions: Vec::new(),
        }
    }

    pub async fn broadcast_json<T: Serialize>(&mut self, value: &T) {
        for session in self.sessions.iter_mut() {
            _ = session.text_json(value).await;
        }
    }

    pub fn permit_access(&mut self, user_id: String, access: AccessLevel) {
        self.allowed_users.insert(user_id, access);
    }

    pub fn get_file_mut(&mut self, file_name: &str) -> Option<&mut Document> {
        self.documents.get_mut(file_name)
    }

    pub fn add_file(&mut self, path: impl Into<String>, document: Document) -> &mut Document {
        let path: String = path.into();
        self.documents.insert(path.clone(), document);

        // SAFETY: just inserted above
        unsafe { self.documents.get_mut(&path).unwrap_unchecked() }
    }

    /// Get all file paths
    pub fn get_files(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    pub fn get_file(&self, path: &str) -> Option<&Document> {
        self.documents.get(path)
    }

    pub fn fork(&self, owner: String) -> Project {
        let name = if self.name.ends_with(" (fork)") {
            self.name.clone()
        } else {
            format!("{} (fork)", self.name)
        };

        let replica_id = generate_unique_replica_id();
        let documents: HashMap<String, Document> = self
            .documents
            .iter()
            .map(|(path, doc)| (path.clone(), doc.fork(replica_id)))
            .collect();

        Project {
            id: Uuid::new_v4(),
            name,
            owner,
            documents,
            allowed_users: HashMap::new(),
            pending_requests: HashMap::new(),
            is_public: self.is_public,
            password: None,
            sessions: Vec::new(),
        }
    }
}

#[derive(Clone)]
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
        let project = Project::new(owner.id.clone(), name);

        self.add_project(project)
    }

    pub fn add_project(&mut self, project: Project) -> &mut Project {
        let project_id = project.id.clone();

        log::info!("New project {}: {}", project.id, project.name);
        self.projects.insert(project_id.clone(), project);

        // SAFETY: the project is just inserted above
        unsafe { self.projects.get_mut(&project_id).unwrap_unchecked() }
    }

    #[expect(dead_code)]
    pub fn get_project(&self, id: &Uuid) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn get_project_mut(&mut self, id: &Uuid) -> Option<&mut Project> {
        self.projects.get_mut(id)
    }
}

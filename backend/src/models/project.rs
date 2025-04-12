use crate::auth::jwt::RgUserData;
use crate::models::document::Document;
use crate::models::file_node::FileNode;
use log::info;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::document::generate_unique_replica_id;
use super::project_access::AccessLevel;

#[derive(Clone)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub documents: HashMap<String, Document>,
    pub allowed_users: HashMap<String, AccessLevel>,
    pub pending_requests: HashSet<String>,
    pub is_public: bool,
    pub password: Option<String>,
}

impl Project {
    pub fn new(owner: String, name: impl Into<String>) -> Self {
        Project {
            id: Uuid::new_v4(),
            name: name.into(),
            owner,
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            pending_requests: HashSet::new(),
            is_public: true,
            password: None,
        }
    }

    pub fn permit_access(&mut self, username: String, access: AccessLevel) {
        self.allowed_users.insert(username, access);
    }

    pub fn get_file_mut(&mut self, file_name: &str) -> Option<&mut Document> {
        self.documents.get_mut(file_name)
    }

    pub fn add_file(&mut self, path: impl Into<String>, document: Document) {
        self.documents.insert(path.into(), document);
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
            files: self.files.clone(),
            allowed_users: HashMap::new(),
            pending_requests: HashSet::new(),
            is_public: self.is_public,
            password: None,
        }
    }
}

#[derive(Clone)]
pub struct ProjectManager {
    projects: HashMap<Uuid, Project>,
}

impl ProjectManager {
    pub fn new() -> Self {
        info!("Inicializando ProjectManager");
        ProjectManager {
            projects: HashMap::new(),
        }
    }

    pub fn new_project(&mut self, owner: &RgUserData, name: impl Into<String>) -> &Project {
        let project = Project::new(owner.id.clone(), name);

        self.add_project(project)
    }

    pub fn add_project(&mut self, project: Project) -> &Project {
        let project_id = project.id.clone();

        log::info!("New project {}: {}", project.id, project.name);
        self.projects.insert(project_id.clone(), project);

        // SAFETY: the project is just inserted above
        unsafe { self.projects.get(&project_id).unwrap_unchecked() }
    }

    pub fn get_project(&self, id: &Uuid) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn get_project_mut(&mut self, id: &Uuid) -> Option<&mut Project> {
        self.projects.get_mut(id)
    }
}

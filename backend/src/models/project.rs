use crate::models::document::Document;
use crate::models::file_node::FileNode;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    ReadOnly,
    Editor,
}

#[derive(Clone)]
pub struct Project {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) owner: String,
    pub(crate) files: FileNode,
    pub(crate) documents: HashMap<String, Document>,
    pub(crate) allowed_users: HashMap<String, AccessLevel>,
    pub(crate) pending_editor_requests: Vec<String>,
    pub(crate) is_public: bool,
    pub(crate) password: Option<String>,
}

impl Project {
    pub fn new(
        owner: String,
        id: Uuid,
        name: impl Into<String>,
        is_public: bool,
        password: Option<String>,
    ) -> Self {
        if !is_public && password.is_none() {
            panic!("Los proyectos privados requieren contraseña");
        }
        Project {
            id,
            name: name.into(),
            owner,
            files: FileNode::Directory(HashMap::new()),
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            pending_editor_requests: Vec::new(),
            is_public,
            password,
        }
    }
    pub fn permit_access(&mut self, username: String, access: AccessLevel) {
        self.allowed_users.insert(username, access);
    }
    pub fn get_file_mut(&mut self, file_name: &str) -> Option<&mut Document> {
        self.documents.get_mut(file_name)
    }

    pub fn add_file(&mut self, path: &str, document: Document) {
        let mut parts: Vec<&str> = path.split('/').collect();
        if let Some(file_name) = parts.pop() {
            let mut current = match &mut self.files {
                FileNode::Directory(dir) => dir,
                _ => panic!("La raíz del proyecto debe ser un directorio"),
            };

            for part in parts {
                current = current
                    .entry(part.to_string())
                    .or_insert_with(|| FileNode::Directory(HashMap::new()))
                    .as_directory_mut()
                    .expect("Se esperaba un directorio");
            }
            current.insert(file_name.to_string(), FileNode::File);
            self.documents.insert(path.to_string(), document);
        }
    }

    pub fn get_files(&self) -> &FileNode {
        &self.files
    }

    pub fn get_file(&self, path: &str) -> Option<&Document> {
        self.documents.get(path)
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

    pub fn add_project(&mut self, project: Project) {
        info!("Agregando proyecto {}", project.id);
        self.projects.insert(project.id, project);
    }

    pub fn get_project(&self, id: Uuid) -> Option<&Project> {
        self.projects.get(&id)
    }

    pub fn get_project_mut(&mut self, id: Uuid) -> Option<&mut Project> {
        self.projects.get_mut(&id)
    }
}

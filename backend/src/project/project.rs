use std::collections::HashMap;

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::collab::{Document, DocumentInfo};
use crate::http_errors::HttpErrors;
use crate::ws::messages::ServerMessage;

use super::AccessLevel;

pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub documents: HashMap<String, Document>,
    pub allowed_users: HashMap<String, AccessLevel>,
    pub is_public: bool,
    pub password: Option<String>,
    pub broadcast: broadcast::Sender<ServerMessage>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            owner: String::new(),
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            is_public: true,
            password: None,
            broadcast: broadcast::channel(u8::MAX as usize).0,
        }
    }
}

impl Project {
    pub fn new(owner: String, name: impl Into<String>) -> Self {
        Project {
            name: name.into(),
            owner,
            ..Default::default()
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

    pub fn rm_file(&mut self, path: impl AsRef<str>) -> Option<Document> {
        self.documents.remove(path.as_ref())
    }

    /// Get all file paths
    pub fn get_files(&self) -> HashMap<String, DocumentInfo> {
        self.documents
            .iter()
            .map(|(path, doc)| (path.clone(), doc.into()))
            .collect()
    }

    pub fn fork(&self, owner: String) -> Project {
        let name = if self.name.ends_with(" (fork)") {
            self.name.clone()
        } else {
            format!("{} (fork)", self.name)
        };

        let documents: HashMap<String, Document> = self
            .documents
            .iter()
            .map(|(path, doc)| (path.clone(), doc.fork()))
            .collect();

        Project {
            name,
            owner,
            documents,
            is_public: self.is_public,
            ..Default::default()
        }
    }

    pub fn join_project(
        &mut self,
        user_id: &String,
        password: Option<String>,
    ) -> Result<AccessLevel, HttpErrors> {
        if let Some(access) = self.allowed_users.get(user_id) {
            Ok(*access)
        } else if !self.is_public {
            Ok(AccessLevel::Queue)
        } else if let Some(ref p_password) = self.password {
            if password.is_none_or(|pass| &pass != p_password) {
                return Err(HttpErrors::InvalidPassword);
            }

            self.permit_access(user_id.clone(), AccessLevel::ReadOnly);
            Ok(AccessLevel::ReadOnly)
        } else {
            self.permit_access(user_id.clone(), AccessLevel::ReadOnly);
            Ok(AccessLevel::ReadOnly)
        }
    }
}

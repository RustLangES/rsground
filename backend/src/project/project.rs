use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::collab::{Document, DocumentInfo};
use crate::http_errors::HttpErrors;
use crate::utils::{ArcStr, AsyncInto, ToStream, EMPTY_STR};
use crate::ws::messages::{InternalMessage, ServerMessage};

use super::AccessLevel;

pub struct Project {
    pub id: Uuid,
    pub name: ArcStr,
    pub owner: ArcStr,
    pub documents: HashMap<ArcStr, Arc<Document>>,
    pub allowed_users: HashMap<ArcStr, AccessLevel>,
    pub requests: HashSet<ArcStr>,
    pub is_public: bool,
    pub password: Option<String>,
    pub internal: broadcast::Sender<InternalMessage>,
    pub broadcast: broadcast::Sender<ServerMessage>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: EMPTY_STR.clone(),
            owner: EMPTY_STR.clone(),
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            requests: HashSet::new(),
            is_public: true,
            password: None,
            internal: broadcast::channel(u8::MAX as usize).0,
            broadcast: broadcast::channel(u8::MAX as usize).0,
        }
    }
}

impl Project {
    pub fn new(owner: ArcStr, name: ArcStr) -> Self {
        Project {
            name,
            owner,
            ..Default::default()
        }
    }

    pub fn add_request(&mut self, user_info: &RgUserData) {
        if self.requests.insert(user_info.id.clone()) {
            _ = self.broadcast.send(ServerMessage::RequestAccess {
                user_id: user_info.id.clone(),
                user_name: user_info.name.clone(),
            });
        }
    }

    pub fn permit_access(&mut self, user_id: ArcStr, access: AccessLevel) {
        self.allowed_users.insert(user_id, access);
    }

    pub fn get_file(&mut self, file_name: &str) -> Option<Arc<Document>> {
        self.documents.get(file_name).cloned()
    }

    pub fn add_file(&mut self, path: impl Into<ArcStr>, document: Document) -> Arc<Document> {
        self.documents
            .entry(path.into())
            .insert_entry(document.into())
            .get()
            .clone()
    }

    pub fn rm_file(&mut self, path: impl AsRef<str>) -> Option<Arc<Document>> {
        self.documents.remove(path.as_ref())
    }

    /// Get all file paths
    pub async fn get_files(&self) -> HashMap<ArcStr, DocumentInfo> {
        (&self.documents)
            .to_stream()
            .map(async |(path, doc)| (path.clone(), doc.async_into().await))
            .buffer_unordered(5)
            .collect()
            .await
    }

    pub async fn fork(&self, owner: ArcStr) -> Project {
        let name = if self.name.ends_with(" (fork)") {
            self.name.clone()
        } else {
            format!("{} (fork)", self.name).as_str().into()
        };

        let documents = (&self.documents)
            .to_stream()
            .map(async |(path, doc)| (path.clone(), doc.fork().await.into()))
            .buffer_unordered(5)
            .collect::<HashMap<ArcStr, Arc<Document>>>()
            .await;

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
        user_id: ArcStr,
        password: Option<String>,
    ) -> Result<AccessLevel, HttpErrors> {
        if let Some(access) = self.allowed_users.get(&user_id) {
            Ok(*access)
        } else if !self.is_public {
            Ok(AccessLevel::Queue)
        } else if let Some(ref p_password) = self.password {
            if password.is_none_or(|pass| &pass != p_password) {
                return Err(HttpErrors::InvalidPassword);
            }

            self.permit_access(user_id, AccessLevel::ReadOnly);
            Ok(AccessLevel::ReadOnly)
        } else {
            self.permit_access(user_id, AccessLevel::ReadOnly);
            Ok(AccessLevel::ReadOnly)
        }
    }
}

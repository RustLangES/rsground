use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use actix::Addr;
use futures::StreamExt;
use rsground_runner::Runner;
use rsground_runner::lsp::notification::{DidChangeTextDocument, DidOpenTextDocument};
use rsground_runner::lsp::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, TextDocumentContentChangeEvent,
    TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::collab::{Document, DocumentInfo};
use crate::http_errors::HttpErrors;
use crate::project::project_lsp::ProjectLspActor;
use crate::utils::{ArcStr, AsyncDefault, AsyncInto, EMPTY_STR, ToStream};
use crate::ws::messages::{InternalMessage, ServerMessage};

use super::{AccessLevel, lsp, producer, project_log};

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
    runner: Arc<Runner>,
    producer: Addr<producer::ProjectProducer>,
    lsp: Addr<lsp::ProjectLsp>,
    lsp_started: bool,
}

impl AsyncDefault for Project {
    async fn default() -> Self {
        let id = Uuid::new_v4();
        let broadcast = broadcast::channel(u8::MAX as usize).0;
        let internal = broadcast::channel(u8::MAX as usize).0;

        let runner = Arc::new(Runner::new().await.expect("Cannot start runner"));

        let producer =
            producer::ProjectProducer::create(id.clone(), broadcast.clone(), runner.clone()).await;

        let lsp = lsp::ProjectLsp::start(id.clone(), internal.clone(), runner.clone()).await;

        Self {
            id,
            name: EMPTY_STR.clone(),
            owner: EMPTY_STR.clone(),
            documents: HashMap::new(),
            allowed_users: HashMap::new(),
            requests: HashSet::new(),
            is_public: true,
            password: None,
            internal,
            broadcast,
            runner,
            producer,
            lsp,
            lsp_started: false,
        }
    }
}

impl Project {
    pub async fn new(owner: ArcStr, name: ArcStr) -> Self {
        Project {
            name,
            owner,
            ..Project::default().await
        }
    }

    pub fn get_runner(&self) -> Arc<Runner> {
        self.runner.clone()
    }

    pub fn get_lsp(&self) -> &Addr<lsp::ProjectLsp> {
        &self.lsp
    }

    pub async fn start_lsp(&mut self) {
        if self.lsp_started {
            return;
        }

        self.lsp_started = true;

        self.lsp.do_send(lsp::Execute);

        for (file, doc) in &self.documents {
            let version = doc.revision().await as i32;
            let text = doc.text().await;

            let Ok(uri) = Uri::from_str(&format!("file:///home/{file}")) else {
                continue;
            };

            self.lsp
                .send_notify::<DidOpenTextDocument, true>(&DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri,
                        language_id: "rust".to_owned(),
                        version,
                        text,
                    },
                });
        }
    }

    pub async fn execute(&self) {
        self.producer.do_send(producer::Execute);
    }

    pub fn stop_execute(&self) {
        self.producer.do_send(producer::Abort);
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

    pub async fn add_file(&mut self, path: impl Into<ArcStr>, document: Document) -> Arc<Document> {
        let path: ArcStr = path.into();
        let content = document.text().await;

        _ = self
            .get_runner()
            .create_file(&path.to_string(), &content)
            .await
            .inspect_err(|err| project_log::error!("{err}"));

        if path.ends_with("rs")
            && let Ok(uri) = Uri::from_str(&format!("file:///home/{path}"))
        {
            self.lsp
                .send_notify::<DidOpenTextDocument, true>(&DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri,
                        language_id: "rust".to_owned(),
                        version: 0,
                        text: content,
                    },
                });
        }

        self.documents
            .entry(path)
            .insert_entry(document.into())
            .get()
            .clone()
    }

    pub fn rm_file(&mut self, path: impl AsRef<str>) -> Option<Arc<Document>> {
        // TODO: (@Brayan-724) Remove real file and notify lsp
        self.documents.remove(path.as_ref())
    }

    pub async fn file_edit(&mut self, path: ArcStr, content: impl AsRef<str>) {
        _ = self
            .runner
            .create_file(&path, content.as_ref())
            .await
            .inspect_err(|err| log::error!("{err}"));

        if let Ok(uri) = Uri::from_str(&format!("file:///home/{path}")) {
            let version = if let Some(doc) = self.documents.get(&path) {
                doc.revision().await as i32
            } else {
                0
            };

            self.lsp
                .send_notify::<DidChangeTextDocument, true>(&DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier { uri, version },
                    content_changes: vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: content.as_ref().to_owned(),
                    }],
                });
        }

        _ = self.internal.send(InternalMessage::FileEdit { path });
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
            ..Project::default().await
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

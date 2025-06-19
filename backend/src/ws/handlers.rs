use std::collections::HashMap;

use actix_ws as ws;
use futures::StreamExt as _;

use crate::collab::Document;
use crate::project::AccessLevel;
use crate::utils::{ArcStr, ToStream};
use crate::ws::messages::{ClientMessage, ServerMessage, ServerMessageError};
use crate::ws::ws_ext::SessionExt;

use super::messages::InternalMessage;
use super::websocket::RgWebsocket;

impl RgWebsocket {
    async fn handle_ws_response(
        ctx: &mut ws::Session,
        msg: Result<ServerMessage, ServerMessageError>,
    ) {
        let response = match msg {
            Ok(ok) => ok,
            Err(ServerMessageError::None) => return,
            Err(err) => err.into(),
        };

        log::trace!("Sending response: {response:#?}");
        _ = ctx.text_json(&response).await;
    }

    pub async fn handle_welcome(&mut self, ctx: &mut ws::Session) {
        // Don't send welcome when is in queue
        if !self.access.can_read() {
            return;
        }

        Self::handle_ws_response(ctx, self.compose_welcome().await).await
    }

    async fn compose_welcome(&mut self) -> Result<ServerMessage, ServerMessageError> {
        let project = self.app_state.get_project(self.project_id).await?;
        let project = project.read().await;

        _ = project.broadcast.send(ServerMessage::UserConnected {
            user_id: self.user_info.id.clone(),
            user_name: self.user_info.name.clone(),
        });

        self.sync_docs = (&project.documents)
            .to_stream()
            .map(async |(k, v)| (k.clone(), (v.clone(), v.revision().await)))
            .buffer_unordered(10)
            .collect()
            .await;

        let files = project.get_files().await;

        let users = (&project.allowed_users)
            .to_stream()
            .filter_map(async |(user, access)| {
                Some((
                    user.clone(),
                    (self.app_state.get_username(user).await?, *access),
                ))
            })
            .collect::<HashMap<ArcStr, (ArcStr, AccessLevel)>>()
            .await;

        let requests = if project.owner == self.user_info.id {
            Some(
                (&project.requests)
                    .to_stream()
                    .filter_map(async |user| {
                        Some((user.clone(), self.app_state.get_username(user).await?))
                    })
                    .collect::<HashMap<ArcStr, ArcStr>>()
                    .await,
            )
        } else {
            None
        };

        Ok(ServerMessage::Welcome {
            session_id: self.session_id.clone(),
            files,
            users,
            requests,
        })
    }

    pub async fn handle_internal(&mut self, msg: InternalMessage, ctx: &mut ws::Session) {
        // Don't send internal updates when is in queue
        if !self.access.can_read() {
            return;
        }

        Self::handle_ws_response(ctx, self.compose_internal(msg).await).await
    }

    async fn compose_internal(
        &mut self,
        msg: InternalMessage,
    ) -> Result<ServerMessage, ServerMessageError> {
        match msg {
            InternalMessage::FileEdit { path } => {
                let Some((doc, revision)) = self.sync_docs.get_mut(&path) else {
                    return Err(ServerMessageError::FileNotFound(path));
                };

                if doc.revision().await > *revision {
                    let (new_revision, actions) = doc.send_history(*revision).await;

                    let msg = if let Some(actions) = actions {
                        Ok(ServerMessage::Sync {
                            file: path,
                            revision: *revision,
                            actions,
                        })
                    } else {
                        Err(ServerMessageError::None)
                    };

                    *revision = new_revision;

                    msg
                } else {
                    Err(ServerMessageError::None)
                }
            }
        }
    }

    pub async fn handle_broadcast(&mut self, msg: ServerMessage, ctx: &mut ws::Session) {
        match msg {
            ServerMessage::UpdateAccess { access, user_id } if user_id == self.user_info.id => {
                self.access = access;

                _ = ctx
                    .text_json(&ServerMessage::UpdateAccess { access, user_id })
                    .await;
            }
            // Only update requests to owner
            ServerMessage::RequestAccess { .. } => {
                if let Ok(p) = self.app_state.get_project(self.project_id).await {
                    if p.read().await.owner == self.user_info.id {
                        _ = ctx.text_json(&msg).await
                    }
                }
            }

            _ if self.access.can_read() => _ = ctx.text_json(&msg).await,
            _ => {}
        }
    }

    pub async fn handle_ws_msg(
        &mut self,
        msg: Result<ws::AggregatedMessage, ws::ProtocolError>,
        ctx: &mut ws::Session,
    ) {
        let Ok(msg) = msg.inspect_err(|e| log::error!("Error in websocket stream: {e:?}")) else {
            return;
        };

        match msg {
            ws::AggregatedMessage::Text(text) => {
                log::trace!("New message: {text}");

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        let msg = self.handle_client_message(ctx, client_msg).await;

                        Self::handle_ws_response(ctx, msg).await;
                    }
                    Err(err) => {
                        log::error!("Could not parse message: {err}");

                        let err = ServerMessage::Error {
                            message: err.to_string(),
                        };
                        _ = ctx.text_json(&err).await;
                    }
                }
            }
            ws::AggregatedMessage::Close(reason) => {
                log::info!("Closed connection: {reason:?}");
                _ = ctx.clone().close(reason).await;
            }
            _ => (),
        }
    }

    async fn handle_client_message(
        &mut self,
        _ctx: &ws::Session,
        msg: ClientMessage,
    ) -> Result<ServerMessage, ServerMessageError> {
        self.access.need_read()?;

        match msg {
            ClientMessage::Config {
                name,
                is_public,
                password,
            } => {
                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                if project.owner != self.user_info.id {
                    return Err(ServerMessageError::None);
                }

                if let Some(name) = name {
                    project.name = name;
                }

                if let Some(is_public) = is_public {
                    project.is_public = is_public;
                }

                if let Some(password) = password {
                    if password.is_empty() {
                        project.password = None;
                    } else {
                        project.password = project.is_public.then_some(password);
                    }
                }

                _ = project.broadcast.send(ServerMessage::ProjectConfig {
                    name: project.name.clone(),
                    is_public: project.is_public,
                    password: project.password.clone(),
                });

                Err(ServerMessageError::None)
            }
            ClientMessage::FileCreate { file } => {
                self.access.need_editor()?;

                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                project.add_file(file, Document::new());

                let msg = ServerMessage::ProjectFiles {
                    files: project.get_files().await,
                };

                _ = project.broadcast.send(msg);

                Err(ServerMessageError::None)
            }
            ClientMessage::FileDelete { file } => {
                self.access.need_editor()?;

                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                if project.rm_file(&file).is_some() {
                    let msg = ServerMessage::ProjectFiles {
                        files: project.get_files().await,
                    };

                    _ = project.broadcast.send(msg);

                    Err(ServerMessageError::None)
                } else {
                    Err(ServerMessageError::FileNotFound(file))
                }
            }
            ClientMessage::PermitAccess { user_id, access } => {
                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                if project.owner != self.user_info.id {
                    return Err(ServerMessageError::NotOwner);
                }

                log::info!("User {user_id} accepted");

                project.permit_access(user_id.clone(), access);

                _ = project
                    .broadcast
                    .send(ServerMessage::UpdateAccess { user_id, access });

                Err(ServerMessageError::None)
            }
            ClientMessage::Sync {
                file,
                revision,
                actions,
            } => {
                self.access.need_editor()?;

                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                let doc = project.get_file(&file).ok_or_else(|| {
                    log::error!("File {file:?} not found in {:?}", self.project_id);
                    ServerMessageError::FileNotFound(file.clone())
                })?;

                if let Err(err) = doc
                    .compose(self.session_id.clone(), revision, actions)
                    .await
                {
                    _ = project
                        .broadcast
                        .send(ServerMessage::Error { message: err })
                }

                dbg!(&doc.text().await);

                _ = project
                    .internal
                    .send(InternalMessage::FileEdit { path: file });

                Err(ServerMessageError::None)
            }

            ClientMessage::SyncCursor { file, cursors } => {
                self.access.need_editor()?;

                let project = self.app_state.get_project(self.project_id).await?;
                let mut project = project.write().await;

                let doc = project.get_file(&file).ok_or_else(|| {
                    log::error!("File {file:?} not found in {:?}", self.project_id);
                    ServerMessageError::FileNotFound(file.clone())
                })?;

                let mut doc = doc.state_mut().await;

                if cursors.is_empty() {
                    doc.cursors.remove(&self.user_info.id);
                } else {
                    doc.cursors.insert(self.user_info.id.clone(), cursors);
                }

                let cursors = doc.cursors.clone();

                _ = project
                    .broadcast
                    .send(ServerMessage::SyncCursors { file, cursors });

                Err(ServerMessageError::None)
            }
            ClientMessage::SyncFiles => {
                self.access.need_read()?;

                let project = self.app_state.get_project(self.project_id).await?;
                let project = project.write().await;

                Ok(ServerMessage::ProjectFiles {
                    files: project.get_files().await,
                })
            }
        }
    }
}

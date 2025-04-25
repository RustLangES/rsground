use actix_ws as ws;

use crate::collab::Document;
use crate::ws::messages::{ClientMessage, ServerMessage, ServerMessageError};
use crate::ws::ws_ext::SessionExt;

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

    pub async fn handle_access_change(&mut self, ctx: &mut ws::Session) {
        let manager = self.app_state.get_manager();

        let Some(project) = manager.get_project(&self.project_id) else {
            return;
        };

        let new_access = project
            .allowed_users
            .get(&self.user_info.id)
            .copied()
            .unwrap_or_default();

        self.access = new_access;

        _ = ctx
            .text_json(&ServerMessage::UpdateAccess {
                access: self.access,
                user_id: self.user_info.id.clone(),
            })
            .await;
    }

    pub async fn handle_welcome(&self, ctx: &mut ws::Session) {
        Self::handle_ws_response(ctx, self.compose_welcome().await).await
    }

    async fn compose_welcome(&self) -> Result<ServerMessage, ServerMessageError> {
        let mut manager = self.app_state.get_manager();
        let project = manager.get_project_mut(self.project_id)?;

        _ = project.broadcast.send(ServerMessage::UserConnected {
            user_id: self.user_info.id.clone(),
        });

        let files = project.get_files().clone();
        let users = project.allowed_users.clone();

        Ok(ServerMessage::Welcome {
            session_id: self.session_id.clone(),
            files,
            users,
        })
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

                let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text)
                    .inspect_err(|err| log::error!("Could not parse message: {err}"))
                else {
                    let err = ServerMessage::Error {
                        message: "Invalid message".into(),
                    };
                    _ = ctx.text_json(&err).await;
                    return;
                };

                let msg = self.handle_client_message(&ctx, client_msg).await;

                Self::handle_ws_response(ctx, msg).await;
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

        let mut manager = self.app_state.get_manager();

        match msg {
            ClientMessage::FileCreate { file } => {
                self.access.need_editor()?;

                let project = manager.get_project_mut(self.project_id)?;

                project.add_file(file, Document::new());

                Err(ServerMessageError::None)
            }
            ClientMessage::FileDelete { file } => {
                self.access.need_editor()?;
                todo!()
            }
            ClientMessage::PermitAccess { user_id, access } => {
                let project = manager.get_project_mut(self.project_id)?;

                if project.owner != self.user_info.id {
                    return Err(ServerMessageError::NotOwner);
                }

                log::info!("User {user_id} accepted");

                project.permit_access(user_id.clone(), access);

                // TODO: Handle access change by broadcast
                if let Some(notifier) = project.pending_requests.remove(&user_id) {
                    notifier.notify_waiters();
                }

                // Update everyone for the new user
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

                let project = manager.get_project_mut(self.project_id)?;

                let doc = project.get_file_mut(&file).ok_or_else(|| {
                    log::error!("File {file:?} not found in {:?}", self.project_id);
                    ServerMessageError::FileNotFound(file.clone())
                })?;

                doc.compose(revision, actions);

                Ok(ServerMessage::Sync {
                    file,
                    revision: doc.revision(),
                    actions: doc.get_history_since(revision),
                })
            }
            ClientMessage::SyncFiles => {
                self.access.need_read()?;

                let project = manager.get_project_mut(self.project_id)?;

                Ok(ServerMessage::ProjectFiles {
                    files: project.get_files().clone(),
                })
            }
        }
    }
}

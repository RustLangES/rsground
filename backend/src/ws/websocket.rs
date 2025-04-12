use std::sync::Arc;

use actix_ws as ws;
use futures::StreamExt;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::models::document::Document;
use crate::models::project_access::{AccessLevel, ProjectAccess};
use crate::state::AppState;

use super::messages::{ClientMessage, ServerMessage, ServerMessageError};
use super::ws_ext::SessionExt;

pub struct RgWebsocket {
    pub app_state: Arc<AppState>,
    pub user_info: RgUserData,
    pub access: ProjectAccess,
}

impl RgWebsocket {
    #[inline(always)]
    fn needs_project(&self) -> Result<&Uuid, ServerMessageError> {
        self.access
            .get_project_id()
            .ok_or(ServerMessageError::NotInProject)
    }

    async fn send_welcome(&self, ctx: &mut ws::Session) {
        _ = ctx
            .text_json(&ServerMessage::UserConnected {
                user_id: self.user_info.id.clone(),
            })
            .await;
    }

    fn handle_client_message(
        &mut self,
        msg: ClientMessage,
    ) -> Result<ServerMessage, ServerMessageError> {
        let mut manager = self.app_state.get_manager();

        match msg {
            ClientMessage::CreateProject { name } => {
                let project = manager.new_project(&self.user_info, name);

                self.access = ProjectAccess::Editor(project.id.clone());

                Ok(ServerMessage::ProjectCreated {
                    project_id: project.id.clone(),
                })
            }
            ClientMessage::JoinProject {
                project_id,
                password,
            } => {
                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                if !project.is_public {
                    project.pending_requests.insert(self.user_info.id.clone());
                    return Err(ServerMessageError::None);
                }

                if let Some(ref p_password) = project.password {
                    if password.is_none_or(|pass| &pass != p_password) {
                        return Err(ServerMessageError::InvalidPassword);
                    }
                }

                self.access = ProjectAccess::ReadOnly(project_id.clone());
                return Ok(ServerMessage::JoinedProject { project_id });
            }

            ClientMessage::GrantEditor { user_id } => {
                let project_id = *self.needs_project()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                if project.owner != self.user_info.id {
                    return Err(ServerMessageError::NotOwner);
                }

                if project.pending_requests.remove(&user_id) {
                    log::info!("Editor mode granted for {user_id} in {project_id}");
                    project.allowed_users.insert(user_id, AccessLevel::ReadOnly);
                    Ok(ServerMessage::EditorGranted { project_id })
                } else {
                    log::info!("The user {user_id} doesn't ask for editor-mode",);
                    Err(ServerMessageError::NobodyAskYou)
                }
            }
            ClientMessage::Insert { file, pos, text } => {
                let project_id = *self.access.need_editor()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                if let Some(doc) = project.get_file_mut(&file) {
                    doc.insert(pos, text);
                    Ok(ServerMessage::Update {
                        project_id,
                        file,
                        content: doc.buffer.clone(),
                    })
                } else {
                    let new_doc = Document::new(text, 1);
                    project.add_file(&file, new_doc);

                    let doc = project.get_file(&file).ok_or(ServerMessageError::None)?;

                    Ok(ServerMessage::Update {
                        project_id,
                        file,
                        content: doc.buffer.clone(),
                    })
                }
            }

            ClientMessage::Delete {
                file,
                range_start,
                range_end,
            } => {
                let project_id = *self.access.need_editor()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                if let Some(doc) = project.get_file_mut(&file) {
                    doc.delete(range_start..range_end);
                    Ok(ServerMessage::Update {
                        project_id,
                        file,
                        content: doc.buffer.clone(),
                    })
                } else {
                    log::error!("File {file:?} not found in {project_id:?}");
                    Err(ServerMessageError::FileNotFound(file))
                }
            }
            ClientMessage::Sync {
                file,
                last_timestamp,
            } => {
                let project_id = *self.needs_project()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                let doc = project.get_file_mut(&file).ok_or_else(|| {
                    log::error!("File {file:?} not found in {project_id:?}");
                    ServerMessageError::FileNotFound(file.clone())
                })?;

                log::info!(
                    "Syncing file {file:?} in {project_id:?} from timestamp {last_timestamp:?}",
                );
                let actions = doc.get_operations_since(last_timestamp);
                Ok(ServerMessage::SyncActions { file, actions })
            }
            ClientMessage::GetProjectFiles => {
                let project_id = *self.needs_project()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                Ok(ServerMessage::ProjectFiles {
                    files: project.get_files().clone(),
                })
            }
            ClientMessage::PermitAccess { username, access } => {
                let project_id = *self.needs_project()?;

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                if project.owner != self.user_info.id {
                    return Err(ServerMessageError::NotOwner);
                }

                project.permit_access(username.clone(), access);
                log::info!("User {username} accepted");

                Ok(ServerMessage::JoinedProject { project_id })
            }
            ClientMessage::ForkProject { project_id } => {
                if self.access.is_reader() {
                    return Err(ServerMessageError::NotAccessible);
                }

                let project = manager
                    .get_project_mut(&project_id)
                    .ok_or_else(|| ServerMessageError::ProjectNotFound(project_id))?;

                let forked_project = project.fork(self.user_info.id.clone());

                let project = manager.add_project(forked_project);

                self.access = ProjectAccess::Editor(project.id.clone());

                Ok(ServerMessage::ProjectForked {
                    project_id: project.id.clone(),
                })
            }
        }
    }

    pub(super) async fn handle(&mut self, msg: ws::AggregatedMessage, ctx: &mut ws::Session) {
        match msg {
            ws::AggregatedMessage::Text(text) => {
                log::trace!("New message: {}", text);
                let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text)
                    .inspect_err(|err| log::error!("Could not parse message: {err}"))
                else {
                    let err = ServerMessage::Error {
                        message: "Invalid message".into(),
                    };
                    _ = ctx.text_json(&err).await;
                    return;
                };

                let response = match self.handle_client_message(client_msg) {
                    Ok(ok) => ok,
                    Err(ServerMessageError::None) => return,
                    Err(err) => err.into(),
                };

                log::trace!("Sending response: {response:#?}");
                _ = ctx.text_json(&response).await;
            }
            ws::AggregatedMessage::Close(reason) => {
                log::info!("Closed connection: {reason:?}");
                _ = ctx.clone().close(reason).await;
            }
            _ => (),
        }
    }

    pub fn start(mut self, mut session: ws::Session, mut stream: ws::AggregatedMessageStream) {
        actix_web::rt::spawn(async move {
            self.send_welcome(&mut session).await;

            while let Some(msg) = stream.next().await {
                let Ok(msg) =
                    msg.inspect_err(|e| log::error!("Error in websocket stream: {:?}", e))
                else {
                    continue;
                };

                self.handle(msg, &mut session).await;
            }
        });
    }
}

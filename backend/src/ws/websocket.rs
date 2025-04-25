use std::sync::Arc;

use actix_ws as ws;
use futures::StreamExt;
use tokio::sync::{broadcast, Notify};
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::http_errors::HttpErrors;
use crate::project::AccessLevel;
use crate::state::AppState;
use crate::ws::ws_ext::SessionExt;

use super::messages::ServerMessage;

pub struct RgWebsocket {
    pub app_state: AppState,
    pub access: AccessLevel,
    pub access_changed: Arc<Notify>,
    pub broadcast: broadcast::Receiver<ServerMessage>,
    pub project_id: Uuid,
    pub session_id: String,
    pub user_info: RgUserData,
}

impl RgWebsocket {
    pub fn join_project(
        app_state: AppState,
        user_info: RgUserData,
        project_id: Uuid,
        password: Option<String>,
    ) -> Result<Self, HttpErrors> {
        let access_changed = Arc::new(Notify::new());

        let (broadcast, access) = {
            let mut manager = app_state.get_manager();
            let Ok(project) = manager.get_project_mut(project_id) else {
                return Err(HttpErrors::ProjectDoesNotExist);
            };

            let broadcast = project.broadcast.clone();

            (
                broadcast.subscribe(),
                project.join_project(&user_info.id, password, access_changed.clone())?,
            )
        };

        let ws = Self {
            app_state,
            access_changed,
            broadcast,
            user_info,
            project_id,
            access,
            session_id: Uuid::new_v4().to_string(),
        };

        Ok(ws)
    }

    pub fn start(mut self, mut session: ws::Session, mut stream: ws::AggregatedMessageStream) {
        actix_web::rt::spawn(async move {
            self.handle_welcome(&mut session).await;

            loop {
                tokio::select! {
                    // TODO: Handle access change by broadcast
                    _ = self.access_changed.notified() => {
                        self.handle_access_change(&mut session).await;
                    },
                    msg = self.broadcast.recv() => {
                        let Ok(msg) = msg else {
                            break;
                        };

                        _ = session.text_json(&msg).await;
                    },
                    msg = stream.next() => {
                        let Some(msg) = msg else {
                            break;
                        };

                        self.handle_ws_msg(msg, &mut session).await;
                    }
                }
            }
        });
    }
}

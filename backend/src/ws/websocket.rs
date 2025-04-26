use actix_ws as ws;
use futures::StreamExt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::jwt::RgUserData;
use crate::http_errors::HttpErrors;
use crate::project::AccessLevel;
use crate::state::AppState;

use super::messages::ServerMessage;

pub struct RgWebsocket {
    pub app_state: AppState,
    pub access: AccessLevel,
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
        let (broadcast, access) = {
            let mut manager = app_state.get_manager();
            let Ok(project) = manager.get_project_mut(project_id) else {
                return Err(HttpErrors::ProjectDoesNotExist);
            };

            let broadcast = project.broadcast.clone();

            (
                broadcast.subscribe(),
                project.join_project(&user_info.id, password)?,
            )
        };

        let ws = Self {
            app_state,
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
                    msg = self.broadcast.recv() => {
                        let Ok(msg) = msg else {
                            break;
                        };

                        self.handle_broadcast(msg, &mut session).await;
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

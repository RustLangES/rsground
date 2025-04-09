use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::auth::handlers::Claims;
use crate::models::document::Action;
use crate::models::project::AccessLevel;
use crate::models::{Document, FileNode, Project};
use crate::{models::document::generate_unique_replica_id, state::AppState};
use actix::{Actor, StreamHandler};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ClientMessage {
    CreateProject {
        project_id: Uuid,
        name: String,
        is_public: bool,
        password: Option<String>,
    },
    JoinProject {
        project_id: Uuid,
        access: AccessLevel,
        password: Option<String>,
    },
    Insert {
        project_id: Uuid,
        file: String,
        pos: usize,
        text: String,
    },
    Delete {
        project_id: Uuid,
        file: String,
        range_start: usize,
        range_end: usize,
    },
    Sync {
        project_id: Uuid,
        file: String,
        last_timestamp: u64,
    },
    GetProjectFiles {
        project_id: Uuid,
    },
    GrantEditor {
        project_id: Uuid,
        user_id: String,
    },
    PermitAccess {
        project_id: Uuid,
        username: String,
        access: AccessLevel,
    },
    ForkProject {
        project_id: Uuid,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ServerMessage {
    UserConnected {
        user_id: String,
    },
    ProjectCreated {
        project_id: Uuid,
    },
    SyncActions {
        project_id: Uuid,
        file: String,
        actions: Vec<Action>,
    },
    Error {
        message: String,
    },
    ProjectFiles {
        project_id: Uuid,
        files: FileNode,
    },
    JoinedProject {
        project_id: Uuid,
    },
    EditorRequestReceived {
        project_id: Uuid,
    },
    NewEditorRequest {
        project_id: Uuid,
        user_id: String,
    },
    EditorGranted {
        project_id: Uuid,
    },
    Update {
        project_id: Uuid,
        file: String,
        content: String,
    },
    ProjectForked {
        old_project_id: Uuid,
        new_project_id: Uuid,
    },
}

struct MyWs {
    state: Arc<AppState>,
    user_id: String, // UUID del usuario o el nombre de usuario de github
    access: Option<AccessLevel>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let welcome_msg = ServerMessage::UserConnected {
            user_id: self.user_id.clone(),
        };
        let text = serde_json::to_string(&welcome_msg).unwrap();
        ctx.text(text);
    }
}
impl MyWs {
    fn handle_client_message(&mut self, msg: ClientMessage) -> Option<ServerMessage> {
        let mut manager = self.state.manager.lock().unwrap_or_else(|e| e.into_inner());
        match msg {
            ClientMessage::CreateProject {
                project_id,
                name,
                is_public,
                password,
            } => {
                let project =
                    Project::new(self.user_id.clone(), project_id, name, is_public, password);
                manager.add_project(project);
                self.access = Some(AccessLevel::Editor);
                Some(ServerMessage::ProjectCreated { project_id })
            }
            ClientMessage::JoinProject {
                project_id,
                access,
                password,
            } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if access == AccessLevel::ReadOnly && !project.is_public {
                        match (&project.password, password) {
                            (Some(ref actual), Some(proporcionada)) if *actual == proporcionada => {
                                self.access = Some(AccessLevel::ReadOnly);
                                return Some(ServerMessage::JoinedProject { project_id });
                            }
                            _ => {
                                return Some(ServerMessage::Error {
                                    message: "Contraseña incorrecta para proyecto privado".into(),
                                });
                            }
                        }
                    }

                    if access == AccessLevel::Editor {
                        let user_str = self.user_id.to_string();
                        if !project.pending_editor_requests.contains(&user_str) {
                            project.pending_editor_requests.push(user_str);
                        }
                        return Some(ServerMessage::EditorRequestReceived { project_id });
                    }

                    self.access = Some(access);
                    return Some(ServerMessage::JoinedProject { project_id });
                }
                Some(ServerMessage::Error {
                    message: format!("Proyecto {} no encontrado", project_id),
                })
            }

            ClientMessage::GrantEditor {
                project_id,
                user_id,
            } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if project.owner != self.user_id {
                        return Some(ServerMessage::Error {
                            message: "No tienes permisos para otorgar acceso de editor".into(),
                        });
                    }
                    if let Some(pos) = project
                        .pending_editor_requests
                        .iter()
                        .position(|id| id == &user_id.to_string())
                    {
                        project.pending_editor_requests.remove(pos);
                        info!("Permiso de editor otorgado a usuario {}", user_id);
                        return Some(ServerMessage::Update {
                            project_id,
                            file: String::from(""),
                            content: format!("Usuario {} ahora tiene permisos de editor", user_id),
                        });
                    } else {
                        info!(
                            "No se encontró la solicitud de editor para el usuario {}",
                            user_id
                        );
                        return Some(ServerMessage::Error {
                            message: "El usuario no tiene una solicitud pendiente".into(),
                        });
                    }
                }
                Some(ServerMessage::Error {
                    message: format!("Proyecto {} no encontrado", project_id),
                })
            }
            ClientMessage::Insert {
                project_id,
                file,
                pos,
                text,
            } => {
                if let Some(AccessLevel::ReadOnly) = self.access {
                    return Some(ServerMessage::Error {
                        message: "Permiso denegado: modo solo lectura".into(),
                    });
                }
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        let _ = doc.insert(pos, text);
                        return Some(ServerMessage::Update {
                            project_id,
                            file,
                            content: doc.buffer.clone(),
                        });
                    } else {
                        let mut new_doc = Document::new("", 1);
                        let _ = new_doc.insert(pos, text);
                        project.add_file(&file, new_doc);
                        if let Some(doc) = project.get_file(&file) {
                            return Some(ServerMessage::Update {
                                project_id,
                                file,
                                content: doc.buffer.clone(),
                            });
                        }
                    }
                }
                Some(ServerMessage::Error {
                    message: format!("Proyecto {} no encontrado", project_id),
                })
            }

            ClientMessage::Delete {
                project_id,
                file,
                range_start,
                range_end,
            } => {
                if let Some(AccessLevel::ReadOnly) = self.access {
                    return Some(ServerMessage::Error {
                        message: "Permiso denegado: modo solo lectura".into(),
                    });
                }
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        let _ = doc.delete(range_start..range_end);
                        return Some(ServerMessage::Update {
                            project_id,
                            file,
                            content: doc.buffer.clone(),
                        });
                    } else {
                        return Some(ServerMessage::Error {
                            message: format!(
                                "Archivo '{}' no encontrado en el proyecto {}",
                                file, project_id
                            ),
                        });
                    }
                }
                Some(ServerMessage::Error {
                    message: format!("Proyecto {} no encontrado", project_id),
                })
            }
            ClientMessage::Sync {
                project_id,
                file,
                last_timestamp,
            } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if let Some(doc) = project.get_file_mut(&file) {
                        info!("Sync: Sincronizando archivo '{}' en proyecto {} a partir del timestamp {}", file, project_id, last_timestamp);
                        let actions = doc.get_operations_since(last_timestamp);
                        return Some(ServerMessage::SyncActions {
                            project_id,
                            file,
                            actions,
                        });
                    } else {
                        error!(
                            "Sync: Archivo '{}' no encontrado en proyecto {}",
                            file, project_id
                        );
                        return Some(ServerMessage::Error {
                            message: format!(
                                "Archivo '{}' no encontrado en el proyecto {}",
                                file, project_id
                            ),
                        });
                    }
                }
                error!("Sync: Proyecto {} no encontrado", project_id);
                Some(ServerMessage::Error {
                    message: format!("Proyecto {} no encontrado", project_id),
                })
            }
            ClientMessage::GetProjectFiles { project_id } => {
                if let Some(project) = manager.get_project(project_id) {
                    Some(ServerMessage::ProjectFiles {
                        project_id,
                        files: project.get_files().clone(), // Retorna la estructura completa
                    })
                } else {
                    Some(ServerMessage::Error {
                        message: format!("Proyecto {} no encontrado", project_id),
                    })
                }
            }
            ClientMessage::PermitAccess {
                project_id,
                username,
                access,
            } => {
                if let Some(project) = manager.get_project_mut(project_id) {
                    if project.owner != self.user_id {
                        return Some(ServerMessage::Error {
                            message: "Solo el creador del proyecto puede conceder permisos".into(),
                        });
                    }
                    project.permit_access(username.clone(), access);
                    info!("Permisos actualizados para el usuario {}", username);

                    Some(ServerMessage::Update {
                        project_id,
                        file: String::new(),
                        content: format!("Permisos actualizados para el usuario {}", username),
                    })
                } else {
                    Some(ServerMessage::Error {
                        message: format!("Proyecto {} no encontrado", project_id),
                    })
                }
            }
            ClientMessage::ForkProject { project_id } => {
                if self.access.is_none() {
                    return Some(ServerMessage::Error {
                        message:
                            "Acceso denegado: se requiere al menos acceso de lectura para forquear"
                                .into(),
                    });
                }

                if let Some(project) = manager.get_project(project_id) {
                    let new_project_id = Uuid::new_v4();
                    let new_replica_id = generate_unique_replica_id(); // Genera un ReplicaId único
                    let forked_documents: HashMap<String, Document> = project
                        .documents
                        .iter()
                        .map(|(path, doc)| (path.clone(), doc.fork(new_replica_id)))
                        .collect();

                    let forked_project = Project {
                        id: new_project_id,
                        name: format!("{} (fork)", project.name),
                        owner: self.user_id.clone(),
                        files: project.files.clone(),
                        documents: forked_documents,
                        allowed_users: HashMap::new(),
                        pending_editor_requests: Vec::new(),
                        is_public: project.is_public,
                        password: project.password.clone(),
                    };

                    manager.add_project(forked_project);
                    self.access = Some(AccessLevel::Editor);
                    return Some(ServerMessage::ProjectForked {
                        old_project_id: project_id,
                        new_project_id,
                    });
                } else {
                    return Some(ServerMessage::Error {
                        message: format!("Proyecto {} no encontrado", project_id),
                    });
                }
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                info!("Mensaje recibido: {}", text);
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let response = self.handle_client_message(client_msg);
                    if let Some(resp) = response {
                        let resp_text = serde_json::to_string(&resp).unwrap();
                        info!("Enviando respuesta: {}", resp_text);
                        ctx.text(resp_text);
                    }
                } else {
                    error!("Error al parsear mensaje JSON");
                    let err = ServerMessage::Error {
                        message: "Mensaje inválido".into(),
                    };
                    let _ = ctx.text(serde_json::to_string(&err).unwrap());
                }
            }
            Ok(ws::Message::Close(reason)) => {
                info!("Cierre de conexión: {:?}", reason);
                ctx.close(reason);
            }
            Err(e) => {
                error!("Error en el stream de WebSocket: {:?}", e);
            }
            _ => (),
        }
    }
}

#[get("/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let token = if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                auth_str.trim_start_matches("Bearer ").to_string()
            } else {
                return HttpResponse::Unauthorized().finish();
            }
        } else {
            return HttpResponse::Unauthorized().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    };

    let secret_key = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(secret_key.as_ref()),
        &jsonwebtoken::Validation::default(),
    );

    let user_id = match token_data {
        Ok(data) => data.claims.sub,
        Err(err) => {
            error!("Error al decodificar JWT: {:?}", err);
            return HttpResponse::Unauthorized().finish();
        }
    };

    let ws = MyWs {
        state: data.get_ref().clone(),
        user_id, // Usamos el "sub" del JWT
        access: None,
    };

    ws::start(ws, &req, stream).expect("Error al iniciar WebSocket")
}

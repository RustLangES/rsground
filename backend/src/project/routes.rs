use actix_error_proc::{proof_route, HttpResult};
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::auth::jwt;
use crate::http_errors::HttpErrors;
use crate::project::AccessLevel;
use crate::state::AppState;

#[proof_route(post("/create/{name}"))]
pub async fn create_project(
    app_state: web::Data<AppState>,
    name: web::Path<String>,
    req: HttpRequest,
) -> HttpResult<HttpErrors> {
    let app_state = app_state.into_inner();
    let name = name.into_inner();

    let user_info = jwt::get_user_info(&req)?;

    let mut manager = app_state.get_manager();
    let project = manager.new_project(&user_info, name);

    project.permit_access(user_info.id, AccessLevel::Editor);

    Ok(HttpResponse::Created().json(json!({
        "id": project.id
    })))
}

#[proof_route(post("/fork/{project_id}"))]
pub async fn fork_project(
    app_state: web::Data<AppState>,
    project_id: web::Path<Uuid>,
    req: HttpRequest,
) -> HttpResult<HttpErrors> {
    let app_state = app_state.into_inner();
    let project_id = project_id.into_inner();

    let user_info = jwt::get_user_info(&req)?;

    let mut manager = app_state.get_manager();
    let Some(project) = manager.get_project(&project_id) else {
        return Err(HttpErrors::ProjectDoesNotExist);
    };

    if !project.allowed_users.contains_key(&user_info.id) {
        return Err(HttpErrors::NotAccessible);
    }

    let forked_project = project.fork(user_info.id.clone());

    let project = manager.add_project(forked_project);

    project.permit_access(user_info.id, AccessLevel::Editor);

    Ok(HttpResponse::Created().json(json!({
        "id": project.id
    })))
}

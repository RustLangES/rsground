mod auth;
mod collab;
mod health;
mod http_errors;
mod project;
mod state;
mod utils;
mod ws;

use std::collections::HashMap;
use std::sync::LazyLock;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use auth::github;
use auth::jwt::JWT_SECRET;
use auth::routes::OAuthData;
use project::ProjectManager;
use state::AppState;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    // Force initialization to verify
    // if env var exists at bootstrap
    LazyLock::force(&JWT_SECRET);

    let oauth_data = web::Data::new(OAuthData {
        client: github::get_oauth_client(),
    });

    log::info!("Iniciando servidor Actix-Web");

    let app_state = web::Data::new(AppState {
        manager: Mutex::new(ProjectManager::new()).into(),
        usernames: Mutex::new(HashMap::new()).into(),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(app_state.clone())
            .app_data(oauth_data.clone())
            .service(health::health)
            .service(auth::routes::auth)
            .service(auth::routes::callback)
            .service(auth::routes::login_guest)
            .service(auth::routes::me)
            .service(auth::routes::update_name)
            .service(project::routes::create_project)
            .service(project::routes::fork_project)
            .service(project::routes::get_project)
            .service(ws::routes::websocket)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

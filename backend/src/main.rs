mod auth;
mod health;
mod http_errors;
mod models;
mod state;
mod utils;
mod ws;

use std::sync::{LazyLock, Mutex};

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use auth::github;
use auth::handlers::OAuthData;
use auth::jwt::JWT_SECRET;
use models::project::ProjectManager;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    // Force initialization to verify if env var exists
    // at bootstrap
    LazyLock::force(&JWT_SECRET);

    let oauth_data = web::Data::new(OAuthData {
        client: github::get_oauth_client(),
    });

    log::info!("Iniciando servidor Actix-Web");

    let app_state = web::Data::new(AppState {
        manager: Mutex::new(ProjectManager::new()).into(),
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
            .service(auth::handlers::oauth)
            .service(auth::handlers::auth_callback)
            .service(auth::handlers::guest_jwt)
            .service(auth::handlers::update_name)
            .service(ws::handlers::websocket)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

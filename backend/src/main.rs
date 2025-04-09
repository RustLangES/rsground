use actix_web::{web, App, HttpServer};
use std::{
    env,
    sync::{Arc, Mutex},
};
mod auth;
mod middleware;
mod models;
mod state;
mod ws;
use auth::{
    auth_callback, guest_jwt,
    handlers::{update_name, OAuthData},
    health, oauth as oauth_routes,
};
use middleware::jwt::JwtMiddleware;
use models::project::ProjectManager;
use state::AppState;
use ws::websocket_handler;

use actix_cors::Cors;
use dotenv::dotenv;
use log::info;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let client_id = ClientId::new(env::var("GITHUB_CLIENT_ID").expect("Falta el client id"));
    let client_secret =
        ClientSecret::new(env::var("GITHUB_CLIENT_SECRET").expect("Falta el client secret"));

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("URL de autorización inválida");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("URL de token inválida");

    let redirect_uri = RedirectUrl::new("http://localhost:3000/auth/callback".to_string())
        .expect("URL de redirección inválida");

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_uri);

    let oauth_data = web::Data::new(OAuthData { client });

    info!("Iniciando servidor Actix-Web");

    let app_state = Arc::new(AppState {
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
            .app_data(web::Data::new(app_state.clone()))
            .app_data(oauth_data.clone())
            .service(health)
            .service(oauth_routes)
            .service(auth_callback)
            .service(guest_jwt)
            .service(
                web::scope("")
                    .wrap(JwtMiddleware)
                    .service(update_name)
                    .service(websocket_handler),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

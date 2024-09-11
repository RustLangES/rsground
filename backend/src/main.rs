use std::io::Result;
use actix_web::{main, App, HttpServer};
use crons::register::register_crons;
use dotenv::dotenv;
use routes::{auth::{auth_callback, auth}, sessions::{post_new_session, run_session_code}};

pub mod routes;
pub mod utils;
pub mod crons;

#[main]
async fn main() -> Result<()> {
    dotenv().ok();

    if let Err(err) = register_crons().await {
        println!("ERROR: Couldn't register crons: {err}");
    }

    HttpServer::new(move || {
        App::new()
            .service(post_new_session)
            .service(run_session_code)
            .service(auth)
            .service(auth_callback)
    })
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}

use std::io::Result;
use actix_web::{main, web::{get, Data}, App, HttpServer};
use routes::{crates::get_crates_list, runner::{get_session_ws, post_new_session, put_session_data}};
use tokio::sync::broadcast::channel;
use utils::{database::init_database, structures::state::AppState};

pub mod routes;
pub mod utils;

#[main]
async fn main() -> Result<()> {
    init_database().await;

    let (session_update_tx, _) = channel(100);
    let data = Data::new( AppState { session_update_tx } );

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(get_crates_list)
            .service(post_new_session)
            .route("/session", get().to(get_session_ws))
            .service(put_session_data)
    })
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}

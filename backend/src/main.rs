use std::io::Result;
use actix_web::{web::get, App, HttpServer};
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::error;
use routes::sessions::session_ws;
use utils::{logger::{format_colored_log, format_log}, runners::Runner};

mod routes;
mod utils;

#[actix_web::main]
async fn main() -> Result<()> {
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .basename("backend")
                .use_timestamp(false)
        )
        .duplicate_to_stdout(Duplicate::Info)
        .format_for_files(format_log)
        .format_for_stdout(format_colored_log)
        .start()
        .unwrap();

    HttpServer::new(|| App::new()
        .route("/session", get().to(session_ws))
    )
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}

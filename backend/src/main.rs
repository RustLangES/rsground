use std::io::Result;
use actix_web::{web::get, App, HttpServer};
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::{info, warn};
use routes::sessions::session_ws;
use utils::{analyzer::analyzer_version, logger::{format_colored_log, format_log}};

mod routes;
mod utils;

#[actix_web::main]
async fn main() -> Result<()> {
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .basename("rsground_backend")
                .use_timestamp(false)
        )
        .duplicate_to_stdout(Duplicate::Info)
        .format_for_files(format_log)
        .format_for_stdout(format_colored_log)
        .start()
        .unwrap();

    if let Some(version) = analyzer_version() {
        info!("Rust analyzer found: {version}");
    } else {
        warn!("Rust analyzer is not found in the system, LSP features will be disabled.");
    }

    HttpServer::new(|| App::new()
        .route("/session", get().to(session_ws))
    )
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}

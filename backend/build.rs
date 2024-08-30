use std::path::Path;
use sqlx::{migrate::{MigrateDatabase, Migrator}, Sqlite, SqlitePool};
use tokio::main;

const DB_URL: &str = "sqlite://data.db";

#[main]
async fn main() {
    if Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        return;
    }

    if Sqlite::create_database(DB_URL).await.is_err() {
        panic!("There was an error while creating the databse.");
    }

    let connection = SqlitePool::connect(DB_URL)
        .await
        .unwrap();

    let results = Migrator::new(Path::new("./migrations"))
        .await
        .unwrap()
        .run(&connection)
        .await;

    if let Err(error) = results {
        panic!("There was an error while migrating: {error}");
    }
}

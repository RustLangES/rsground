use std::{env::var, path::Path};
use dotenv::dotenv;
use sqlx::{migrate::{MigrateDatabase, Migrator}, Sqlite, SqlitePool};
use tokio::main;

#[main]
async fn main() {
    dotenv().ok();

    let db_url = &var("DATABASE_URL")
        .expect("Cannot run migrations without DATABASE_URL environment variable.");

    if Sqlite::database_exists(db_url).await.unwrap_or(false) {
        return;
    }

    if let Err(err) = Sqlite::create_database(db_url).await {
        panic!("There was an error while creating the databse: {err}");
    }

    let connection = SqlitePool::connect(db_url)
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

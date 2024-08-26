use std::path::Path;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Pool,
    Sqlite,
    SqlitePool
};
use crate::db;

const DB_URL: &str = "sqlite://data.db";

pub async fn init_database() {
    if Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        return;
    }

    if Sqlite::create_database(DB_URL).await.is_err() {
        panic!("There was an error while creating the databse.");
    }

    let results = Migrator::new(Path::new("./migrations"))
        .await
        .unwrap()
        .run(db!())
        .await;

    if let Err(error) = results {
        panic!("There was an error while migrating: {error}")
    }
}

pub async fn get_db_connection() -> Pool<Sqlite> {
    SqlitePool::connect(DB_URL).await.unwrap()
}

#[macro_export]
macro_rules! db {
    () => {
        &$crate::utils::database::get_db_connection().await
    };
}

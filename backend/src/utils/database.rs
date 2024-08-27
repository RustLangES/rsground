use std::{path::Path, sync::OnceLock};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Pool,
    Sqlite,
    SqlitePool
};

const DB_URL: &str = "sqlite://data.db";
static DB_CELL: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn get_db_connection<'r>() -> &'r Pool<Sqlite> {
    if let Some(connection) = DB_CELL.get() {
        return connection;
    }

    if Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        let connection = SqlitePool::connect(DB_URL).await.unwrap();
        return DB_CELL.get_or_init(|| connection);
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

    DB_CELL.get_or_init(|| connection)
}

#[macro_export]
macro_rules! db {
    () => {
        $crate::utils::database::get_db_connection().await
    };
}

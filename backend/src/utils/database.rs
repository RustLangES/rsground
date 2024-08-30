use std::sync::OnceLock;
use sqlx::{Pool, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://data.db";
static DB_CELL: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn get_db_connection<'r>() -> &'r Pool<Sqlite> {
    if let Some(connection) = DB_CELL.get() {
        return connection;
    }
 
    let connection = SqlitePool::connect(DB_URL)
        .await
        .unwrap();

    DB_CELL.get_or_init(|| connection)
}

#[macro_export]
macro_rules! db {
    () => {
        $crate::utils::database::get_db_connection().await
    };
}

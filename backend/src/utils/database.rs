use std::{env::var, sync::OnceLock};
use sqlx::{Pool, Sqlite, SqlitePool};

static DB_CELL: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn get_db_connection<'r>() -> &'r Pool<Sqlite> {
    if let Some(connection) = DB_CELL.get() {
        return connection;
    }
 
    let connection = SqlitePool::connect(&var("DATABASE_URL").unwrap())
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

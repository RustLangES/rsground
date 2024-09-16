use sqlx::{query, Sqlite};

use crate::{db, utils::docker::modules::{delete_docker_session, ContainerRetrivalError}};

pub async fn clear_containers() {
    let expired = query!("SELECT hash FROM runners WHERE strftime('%Y-%m-%d %H:%M:%S', expires_at) <= datetime('now')")
        .fetch_all(db!())
        .await;

    if expired.is_err() {
        println!("ERROR: Couldn't clear containers, database error.");
        return;
    }

    let mut deleted = Vec::new();

    for runner in expired.unwrap() {
        match delete_docker_session(&runner.hash).await {
            Err(ContainerRetrivalError::Error) => {
                println!("ERROR: There was an error while deleting runner {}.", runner.hash);
            },
            _ => {
                println!("INFO: Deleted runner {}.", runner.hash);
                deleted.push(runner.hash);
            }
        };
    }

    let delete_query = format!("DELETE FROM runners WHERE hash in ({})", vec!["?"; deleted.len()].join(", "));
    let mut delete_query = query::<Sqlite>(&delete_query);

    for hash in &deleted {
        delete_query = delete_query
            .bind(hash);
    }

    match delete_query.execute(db!()).await {
        Ok(result) if result.rows_affected() > 0 => {
            println!("INFO: Deleted {} runner registers agnostic to container existence.", result.rows_affected());
        },
        Err(_) => {
            println!("ERROR: Couldn't delete runner registers.");
        },
        _ => {}
    }
}

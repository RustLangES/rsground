use std::ops::Add;
use chrono::Duration;
use sqlx::types::chrono::Utc;
use futures_util::StreamExt;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use shiplift::{tty::TtyChunk, ContainerOptions, Docker, ExecContainerOptions, ImageListOptions, PullOptions};
use sqlx::query;
use crate::db;

fn generate_hash() -> String {
    let mut rng = thread_rng();

    (0..8)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub async fn create_docker_session() -> Option<String> {
    let hash = generate_hash();
    let docker = Docker::new();

    let rust_image_exists = docker.images()
        .list(&ImageListOptions::default())
        .await
        .unwrap_or_default()
        .iter()
        .any(|img| img
            .repo_tags
            .clone()
            .map_or(false, |tags| tags
                .contains(&"rust:latest".to_string())
            )
        );

    if !rust_image_exists {
        let mut stream = docker.images()
            .pull(
                &PullOptions::builder()
                    .image("rust:latest")
                    .build()
            );

        while let Some(status) = stream.next().await {
            if status.is_err() {
                return None;
            }
        }
    }

    docker.containers()
        .create(
            &ContainerOptions::builder("rust:latest")
                .name(&format!("runner_{hash}"))
                .cmd(vec!["bash", "-c", "cargo new host --bin"])
                .cmd(vec!["sleep", "infinity"])
                .attach_stdin(true)
                .attach_stdout(true)
                .attach_stderr(true)
                .tty(true)
                .build()
        )
        .await
        .ok()?;

    docker.containers()
        .get(&format!("runner_{hash}"))
        .start()
        .await
        .ok()?;

    let expiration = Utc::now().add(Duration::hours(3));

    let insert_is_err = query!(
        "INSERT INTO sessions(hash, name, expires_at) VALUES(?, 'untitled', ?)",
        hash,
        expiration
    )
        .execute(db!())
        .await
        .is_err();

    if insert_is_err {
        docker.containers()
            .get(&format!("runner_{hash}"))
            .delete()
            .await
            .ok()?;

        return None;
    }

    Some(hash)
}

pub async fn get_container_file(hash: String, path: impl ToString) -> Option<String> {
    let docker = Docker::new();

    let mut exec = docker.containers()
        .get(&format!("runner_{hash}"))
        .exec(
            &ExecContainerOptions::builder()
                .attach_stdout(true)
                .attach_stderr(true)
                .cmd(vec!["cat", &path.to_string()])
                .build()
        );

    let mut output = Vec::new();

    while let Some(chunk) = exec.next().await {
        match chunk {
            Ok(TtyChunk::StdOut(data)) => {
                output.extend(data);
            },
            _ => return None
        }
    }

    String::from_utf8(output).ok()
}

pub async fn set_container_file(hash: String, path: impl ToString, contents: String) -> Option<()> {
    let docker = Docker::new();

    let mut exec = docker.containers()
        .get(&format!("runner_{hash}"))
        .exec(
            &ExecContainerOptions::builder()
                .cmd(vec!["bash", "-c", &format!(
                    "echo \"{}\" > {}",
                    contents.replace("\"", "\\\""),
                    path.to_string()
                )])
                .attach_stdout(true)
                .attach_stderr(true)
                .build()
        );

    while let Some(chunk) = exec.next().await {
        match chunk {
            Ok(TtyChunk::StdOut(_)) => {},
            _ => { return None; }
        }
    };

    Some(())
}

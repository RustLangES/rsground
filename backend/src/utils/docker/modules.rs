use std::ops::Add;
use actix_web::HttpResponse;
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use shiplift::{ContainerOptions, Error, ExecContainerOptions, ImageListOptions, PullOptions};
use sqlx::query;
use crate::{db, get_docker};

use super::abstractions::{run_container_command, CCmdOutputString, DockerCommand};

pub enum ContainerRetrivalError {
    Error,
    NotFound
}

impl ContainerRetrivalError {
    pub fn to_http_response(&self) -> HttpResponse {
        match self {
            ContainerRetrivalError::Error
                => HttpResponse::InternalServerError()
                    .into(),
            ContainerRetrivalError::NotFound
                => HttpResponse::NotFound()
                    .content_type("text/utf8")
                    .body("Runner not found.")
        }
    }
}

pub async fn create_docker_session() -> Option<String> {
    let docker = get_docker!();
    let mut rng = thread_rng();
    let hash = (0..16)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    let name = format!("runner_{hash}");

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
                .name(&name)
                .cmd(vec!["sleep", "infinity"])
                .build()
        )
        .await
        .ok()?;

    docker.containers()
        .get(&name)
        .start()
        .await
        .ok()?;

    docker.containers()
        .get(&name)
        .exec(
            &ExecContainerOptions::builder()
                .cmd(vec!["cargo", "new", "host", "--bin"])
                .build()
        )
        .next()
        .await;

    let expiration = Utc::now().add(Duration::seconds(60));

    let insert_is_err = query!(
        "INSERT INTO runners(hash, expires_at) VALUES(?, ?)",
        hash,
        expiration
    )
        .execute(db!())
        .await
        .is_err();

    if insert_is_err {
        docker.containers()
            .get(&name)
            .delete()
            .await
            .ok()?;

        return None;
    }

    Some(hash)
}

pub async fn delete_docker_session<T: ToString + ?Sized>
(hash: &T) -> Result<(), ContainerRetrivalError> {
    let container = get_docker!()
        .containers()
        .get(format!("runner_{}", hash.to_string()));

    container
        .stop(None)
        .await
        .map_err(|err| match err {
            Error::Fault { code, .. } if code == 404
                => ContainerRetrivalError::NotFound,
            _ => ContainerRetrivalError::Error
        })?;

    container
        .delete()
        .await
        .map_err(|_| ContainerRetrivalError::Error)
}

pub async fn run_container_code<T: ToString + ?Sized>
(hash: &T) -> Result<String, ContainerRetrivalError> {
    run_container_command(
        hash,
        &DockerCommand::new()
            .cmd(vec!["cargo", "run", "--manifest-path", "/host/Cargo.toml"])
    )
        .await
        .and_then(|output| output
            .to_string()
            .ok_or(ContainerRetrivalError::Error)
        )
}

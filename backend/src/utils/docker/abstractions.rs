use std::sync::OnceLock;
use futures_util::StreamExt;
use shiplift::{tty::TtyChunk, Docker, Error, ExecContainerOptions};
use super::modules::ContainerRetrivalError;

pub static DOCKER_CONN: OnceLock<Docker> = OnceLock::new();

#[macro_export]
macro_rules! get_docker {
    () => {{
        $crate::utils::docker::abstractions::DOCKER_CONN
            .get_or_init(|| shiplift::Docker::new())
    }};
}

pub enum ContainerCommandOutput {
    StdOut(Vec<u8>),
    StdErr(Vec<u8>)
}

pub trait CCmdOutputString {
    fn to_string(&self) -> Option<String>;
}

impl CCmdOutputString for Vec<ContainerCommandOutput> {
    fn to_string(&self) -> Option<String> {
        String::from_utf8(
            self
                .iter()
                .flat_map(|output| match output {
                    ContainerCommandOutput::StdOut(data)
                    | ContainerCommandOutput::StdErr(data)
                    => data
                }).copied()
                .collect()
        )
            .ok()
    }
}

#[derive(Clone)]
pub struct DockerCommand {
    commands: Vec<Vec<String>>
}

impl DockerCommand {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn cmd(mut self, command: Vec<impl ToString>) -> Self {
        self.commands.push(
            command
                .iter()
                .map(ToString::to_string)
                .collect()
        );
        self
    }

    pub fn build(self) -> Vec<ExecContainerOptions> {
        let mut options = Vec::new();

        for command in self.commands {
            let exec = ExecContainerOptions::builder()
                .cmd(
                    command
                        .iter()
                        .map(String::as_str)
                        .collect()
                )
                .attach_stdout(true)
                .attach_stderr(true)
                .build();

            options.push(exec);
        }

        options
    }
}

impl Default for DockerCommand {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_container_command<T: ToString + ?Sized>
(hash: &T, runner: &DockerCommand)
-> Result<Vec<ContainerCommandOutput>, ContainerRetrivalError> {
    let container = get_docker!()
        .containers()
        .get(&format!("runner_{}", hash.to_string()));

    let mut output: Vec<ContainerCommandOutput> = Vec::new();

    for option in runner.clone().build() {
        let mut exec = container.exec(&option);

        while let Some(chunk) = exec.next().await {
            match chunk {
                Ok(TtyChunk::StdOut(data)) => {
                    output.push(ContainerCommandOutput::StdOut(data));
                },
                Ok(TtyChunk::StdErr(data)) => {
                    output.push(ContainerCommandOutput::StdErr(data));
                },
                Err(Error::Fault { code, .. }) if code == 404 => {
                    return Err(ContainerRetrivalError::NotFound);
                },
                _ => {
                    return Err(ContainerRetrivalError::Error);
                }
            }
        };
    }

    Ok(output)
}

pub async fn get_container_file<T: ToString + ?Sized>
(hash: &T, path: impl ToString) -> Result<String, ContainerRetrivalError> {
    run_container_command(
        hash,
        &DockerCommand::new()
            .cmd(vec![
                "cat",
                &path.to_string()
            ])
    )
        .await
        .and_then(|output| output
            .to_string()
            .ok_or(ContainerRetrivalError::Error)
        )
}

pub async fn set_container_file<T: ToString + ?Sized>
(hash: &T, path: impl ToString, contents: &T)
-> Result<(), ContainerRetrivalError> {
    run_container_command(
        &hash.to_string(),
        &DockerCommand::new()
            .cmd(vec![
                "bash",
                "-c",
                &format!(
                    "echo \"{}\" > {}",
                    contents.to_string().replace("\"", "\\\""),
                    path.to_string()
                )
            ])
    )
        .await
        .map(|_| ())
}

use std::{env::current_exe, fs::{create_dir_all, remove_dir_all}, path::PathBuf, process::{Command, Stdio}, io::Error as IoError};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerCreateError {
    #[error("Couldn't get executable path: {0}")]
    Executable(String),

    #[error("Couldn't get executable path parent.")]
    Parent,

    #[error("Couldn't create context folder: {0}")]
    Context(String),

    #[error("Couldn't create cargo project: {0}")]
    Project(String)
}

#[derive(Error, Debug)]
pub enum RunnerDeleteError {
    #[error("Could not delete the runner files: {0}")]
    DeleteFiles(#[from] IoError)
}

pub struct Runner {
    hash: String,
    path: PathBuf
}

impl Runner {
    pub fn create() -> Result<Self, RunnerCreateError> {
        let hash = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();

        let runners_path = current_exe()
            .map_err(|err| RunnerCreateError::Executable(err.to_string()))?
            .parent()
            .ok_or(RunnerCreateError::Parent)?
            .join("runners/");

        if !runners_path.exists() {
            create_dir_all(&runners_path)
                .map_err(|err| RunnerCreateError::Context(err.to_string()))?;
        }

        let path = runners_path
            .join(format!("runner_{hash}"));

        Command::new("cargo")
            .args(["new", "--bin", &path.display().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|err| RunnerCreateError::Project(err.to_string()))?;

        Ok(Self {
            hash,
            path
        })
    }

    pub fn delete(&self) -> Result<(), RunnerDeleteError> {
        Ok(remove_dir_all(&self.path)?)
    }

    pub fn hash(&self) -> &String {
        &self.hash
    }
}

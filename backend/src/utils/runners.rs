use std::{env::current_exe, fs::{create_dir_all, remove_dir_all}, path::PathBuf, process::{Command, Stdio}};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use thiserror::Error;
use log::error;

#[derive(Error, Debug)]
pub enum RunnerCreateError {
    #[error("Couldn't reach the target path while creating a project: {internal:?}")]
    IoError {
        internal: String
    },
    #[error("A internal cargo error occurred while creating a project: {internal:?}")]
    CargoError {
        internal: String
    }
}

#[derive(Error, Debug)]
pub enum RunnerDeleteError {
    #[error("Could not delete the runner files: {internal:?}")]
    IoError {
        internal: String
    }
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
            .map_err(|err| RunnerCreateError::IoError {
                internal: err.to_string()
            })?
            .parent()
            .ok_or(
                RunnerCreateError::IoError {
                    internal: "Couldn't get path parent.".to_string()
                }
            )?
            .join("runners/");

        if !runners_path.exists() {
            create_dir_all(&runners_path)
                .map_err(|err| RunnerCreateError::IoError {
                    internal: err.to_string()
                })?;
        }

        let path = runners_path
            .join(format!("runner_{hash}"));

        Command::new("cargo")
            .args(["new", "--bin", &path.display().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|err| RunnerCreateError::CargoError {
                internal: err.to_string()
            })?;

        Ok(Self {
            hash,
            path
        })
    }

    pub fn delete(&self) -> Result<(), RunnerDeleteError> {
        remove_dir_all(&self.path)
            .map_err(|err| RunnerDeleteError::IoError {
                internal: err.to_string()
            })
    }

    pub fn hash(self) -> String {
        self.hash
    }
}

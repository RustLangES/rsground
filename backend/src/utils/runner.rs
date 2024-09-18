use std::{collections::HashMap, env::current_exe, fs::{create_dir_all, remove_dir_all, OpenOptions}, io::{Error as IoError, Write}, path::PathBuf, process::{Command, Stdio}};
use log::error;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use thiserror::Error;
use toml::from_str;

use super::communication::{ActError, RequestActor, RunnerRequestOp};

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

#[derive(Error, Debug)]
pub enum RunnerUpdateError {
    #[error("Could not update the file")]
    UpdateFile(#[from] IoError),

    #[error("The passed TOML string is not valid or not allowed.")]
    InvalidPackageString
}

// TODO: Implement another way to check the package string
// it must only be the [packages] section on the Cargo.toml.

#[derive(Deserialize)]
struct Dependencies {
    #[serde(flatten)]
    packages: HashMap<String, Package>
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Package {
    Simple(String),
    Detailed { version: String, features: Option<Vec<String>> }
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

    fn update_internal_file
    (&self, extension: impl ToString, contents: &str)
    -> Result<(), RunnerUpdateError> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path.join(extension.to_string()))?
            .write_all(contents.as_bytes())?;

        Ok(())
    }

    pub fn update_code(&self, code: &str) -> Result<(), RunnerUpdateError> {
        self.update_internal_file("src/main.rs", code)
    }

    pub fn update_packages(&self, code: &str) -> Result<(), RunnerUpdateError> {
        let base_cargo_toml = format!(
            r#"
                [package]
                name = "runner_{}"
                version = "0.1.0"
                edition = "2021"

            "#,
            self.hash()
        );

        if from_str::<Dependencies>(code).is_err() {
            return Err(RunnerUpdateError::InvalidPackageString)
        }

        self.update_internal_file("Cargo.toml", &(base_cargo_toml + code))
    }
}

impl RequestActor for Runner {
    type ContentType = String;

    fn act(&self, op: &RunnerRequestOp, content: &Option<Self::ContentType>)
    -> Result<(), ActError> {
        // TODO: implement run code.

        let content = match content {
            Some(content) => content,
            None => return Err(ActError::MissingContent)
        };

        match op {
            RunnerRequestOp::UploadCode => {
                self.update_code(content)
                    .map_err(|err| {
                        error!("{err:?}");
                        ActError::InternalServerError
                    })
            },
            RunnerRequestOp::UpdateCargo => {
                self.update_packages(content)
                    .map_err(|err|
                        match err {
                            RunnerUpdateError::UpdateFile(err) => {
                                error!("{err:?}");
                                ActError::InternalServerError
                            },
                            RunnerUpdateError::InvalidPackageString
                                => ActError::InvalidToml
                        }
                    )
            },
            _ => { unreachable!() }
        }
    }
} 

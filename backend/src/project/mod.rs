#[allow(clippy::module_inception)]
mod project;
mod project_access;
mod project_manager;
mod project_producer;
mod project_runner;
pub mod routes;

pub use project::Project;
pub use project_access::AccessLevel;
pub use project_manager::ProjectManager;
pub(self) use project_producer as producer;

use crate::utils::define_local_logger;

define_local_logger! {project_log as "backend::project" {}}

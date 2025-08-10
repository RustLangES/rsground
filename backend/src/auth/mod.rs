pub mod github;
pub mod jwt;
pub mod routes;

use crate::utils::define_local_logger;

define_local_logger! {auth_log as "backend::auth" {
    callback,
    jwt,
}}

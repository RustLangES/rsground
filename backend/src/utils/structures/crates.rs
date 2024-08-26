use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;


#[derive(Serialize, Deserialize)]
pub struct Crate {
    pub name: String,
    pub description: String,
    pub downloads: u32,
    #[serde(rename(deserialize = "newest_version"))]
    pub version: String,
    pub id: String
}

#[derive(Deserialize)]
pub struct CratesResponse {
    pub crates: Vec<Crate>
}

#[derive(Debug, Deserialize)]
pub struct CargoToml {
    dependencies: Option<BTreeMap<String, Value>>
}

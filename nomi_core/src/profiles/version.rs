use serde::{Deserialize, Serialize};

use crate::config::toml::TomlConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionProfile {
    id: i32,
    pub version: String,
    pub version_type: String,
    pub path: String,
    pub name: String,
    pub is_downloaded: bool,
}

pub type VersionProfileConfig = TomlConfig<VersionProfile>;

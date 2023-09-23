use serde::{Deserialize, Serialize};

use crate::config::toml::TomlConfig;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VersionProfile {
    pub id: i32,
    pub version: String,
    pub version_type: String,
    pub path: String,
    pub name: String,
    pub is_downloaded: bool,
}

pub type VersionProfileConfig = TomlConfig<VersionProfile>;

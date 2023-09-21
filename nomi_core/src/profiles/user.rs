use serde::{Deserialize, Serialize};

use crate::config::toml::TomlConfig;

use super::version::VersionProfile;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub profiles: Vec<VersionProfile>,
}

pub type UserConfig = TomlConfig<User>;

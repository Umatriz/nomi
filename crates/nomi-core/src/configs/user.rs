use serde::{Deserialize, Serialize};

use crate::repository::{java_runner::JavaRunner, username::Username};

/// `Settings` its a global settings of the launcher
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub username: Username,
    pub access_token: Option<String>,
    pub java_bin: Option<JavaRunner>,
    pub uuid: Option<String>,
}

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::fs::{read_toml_config, write_toml_config};

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mock = Settings {
            username: Username::new("test").unwrap(),
            access_token: Some("access_token".into()),
            java_bin: Some(JavaRunner::default()),
            uuid: Some("uuid".into()),
        };

        write_toml_config(&mock, "./.nomi/User.toml").await.unwrap();
    }

    #[tokio::test]
    async fn read_test() {
        let data: Settings = read_toml_config("./configs/User.toml").await.unwrap();

        dbg!(data);
    }
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::version::VersionProfile;

/// `Settings` its a global settings of the launcher
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub usernames: String,
    pub access_token: Option<String>,
    pub java_bin: Option<PathBuf>,
    pub uuid: Option<String>,
    pub profiles: Vec<VersionProfile>,
}

impl Settings {
    /// Expects a mutable reference to `Self`
    /// creates new `VersionProfile` and pushes it to the `profiles` field
    pub fn add_profile(&mut self, profile: VersionProfile) {
        self.profiles.push(profile)
    }

    /// create id for profile
    /// depends on the last id in the vector
    pub fn create_id(&self) -> i32 {
        match &self.profiles.iter().max_by_key(|x| x.id) {
            Some(v) => v.id + 1,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::profiles::{read_config, version::VersionProfileBuilder, write_config};

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mut mock = Settings {
            usernames: "test".to_owned(),
            profiles: vec![],
            access_token: Some("access_token".into()),
            java_bin: Some("./java/bin/java.exe".into()),
            uuid: Some("uuid".into()),
        };
        let profile = VersionProfileBuilder::new()
            .id(mock.create_id())
            .assets("./minecraft/assets".into())
            .game_dir("./minecraft".into())
            .is_downloaded(false)
            .libraries_dir("./minecraft/libraries".into())
            .manifest_file("./minecraft/versions/1.20/1.20.json".into())
            .natives_dir("./minecraft/versions/1.20/natives".into())
            .version("1.20".into())
            .version_jar_file("./minecraft/versions/1.20/1.20.jar".into())
            .version_type("release".into())
            .build();
        mock.add_profile(profile);
        let profile2 = VersionProfileBuilder::new()
            .id(mock.create_id())
            .assets("./minecraft/assets".into())
            .game_dir("./minecraft".into())
            .is_downloaded(false)
            .libraries_dir("./minecraft/libraries".into())
            .manifest_file("./minecraft/versions/1.20/1.20.json".into())
            .natives_dir("./minecraft/versions/1.20/natives".into())
            .version("1.20".into())
            .version_jar_file("./minecraft/versions/1.20/1.20.jar".into())
            .version_type("release".into())
            .build();
        mock.add_profile(profile2);
        let profile3 = VersionProfileBuilder::new()
            .id(mock.create_id())
            .assets("./minecraft/assets".into())
            .game_dir("./minecraft".into())
            .is_downloaded(false)
            .libraries_dir("./minecraft/libraries".into())
            .manifest_file("./minecraft/versions/1.20/1.20.json".into())
            .natives_dir("./minecraft/versions/1.20/natives".into())
            .version("1.20".into())
            .version_jar_file("./minecraft/versions/1.20/1.20.jar".into())
            .version_type("release".into())
            .build();
        mock.add_profile(profile3);

        write_config(&mock, "./configs/Settings.toml")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn read_test() {
        let data: Settings = read_config("./configs/Settings.toml").await.unwrap();

        dbg!(data);
    }
}

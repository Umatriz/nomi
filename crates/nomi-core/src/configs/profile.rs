use const_typed_builder::Builder;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    instance::{
        launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettingsBuilder},
        profile::{self, read, Profile},
    },
    repository::{java_runner::JavaRunner, username::Username},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VersionProfilesConfig {
    pub profiles: Vec<VersionProfile>,
}

impl VersionProfilesConfig {
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

#[derive(Serialize, Deserialize, Debug, Default, Builder)]
pub struct VersionProfile {
    pub id: i32,
    pub is_downloaded: bool,
    pub name: String,

    pub version: String,
    pub version_type: String,
    pub version_jar_file: PathBuf,

    pub assets: PathBuf,
    pub game_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub profile_file: Option<PathBuf>,
    pub natives_dir: PathBuf,
}

impl VersionProfile {
    pub fn into_launch(
        self,
        username: Username,
        java_bin: JavaRunner<'static>,
        access_token: Option<String>,
        uuid: Option<String>,
    ) -> LaunchInstance<'static> {
        let settings = LaunchSettingsBuilder::new()
            .game_dir(self.game_dir)
            .assets(self.assets)
            .libraries_dir(self.libraries_dir)
            .manifest_file(self.manifest_file)
            .natives_dir(self.natives_dir)
            .version(self.version)
            .version_jar_file(self.version_jar_file)
            .version_type(self.version_type)
            .username(username)
            .access_token(access_token)
            .java_bin(java_bin)
            .uuid(uuid)
            .build();

        let builder = LaunchInstanceBuilder::new().settings(settings);

        // Ok(match self.profile_file {
        //     Some(path) => {
        //         let profile = read(path).await?;
        //     }
        //     None => builder.build(),
        // })
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::write_config;

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mut mock = VersionProfilesConfig { profiles: vec![] };
        let profile = VersionProfileBuilder::new()
            .id(mock.create_id())
            .name("Minecraft".into())
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
            .name("Minecraft".into())
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
            .name("Minecraft".into())
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

        write_config(&mock, "./configs/Profiles.toml")
            .await
            .unwrap();
    }
}

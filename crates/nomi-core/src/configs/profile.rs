use const_typed_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::instance::launch::LaunchInstance;

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

/*
// TODO: add `profile` field that contains an enum of supported profiles
// TODO: cleanup names issues in `instance::profile` and `configs::profile`
// TODO: fix `into_launch`
*/

#[derive(Serialize, Deserialize, Debug, Default, Builder, Clone)]
pub struct VersionProfile {
    pub id: i32,
    pub is_downloaded: bool,
    pub name: String,

    pub instance: LaunchInstance,
}

impl VersionProfile {
    pub async fn launch(&self) -> anyhow::Result<()> {
        self.instance.launch().await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        configs::write_toml_config,
        instance::{launch::LaunchSettings, InstanceBuilder},
        loaders::fabric::Fabric,
        repository::{java_runner::JavaRunner, username::Username},
    };

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mut mock = VersionProfilesConfig { profiles: vec![] };

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let builder = InstanceBuilder::new()
            .version("1.20".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/versions/1.20".into())
            .instance(Box::new(Fabric::new("1.20", None::<String>).await.unwrap()))
            // .instance(Inner::vanilla("1.20").await.unwrap())
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.20-fabric-test".into())
            .sender(tx)
            .build();

        let mc_dir = std::env::current_dir().unwrap().join("minecraft");
        let settings = LaunchSettings {
            access_token: None,
            username: Username::new("ItWorks").unwrap(),
            uuid: None,
            assets: mc_dir.join("assets"),
            game_dir: mc_dir.clone(),
            java_bin: JavaRunner::default(),
            libraries_dir: mc_dir.clone().join("libraries"),
            manifest_file: mc_dir.clone().join("versions/1.18.2/1.18.2.json"),
            natives_dir: mc_dir.clone().join("versions/1.18.2/natives"),
            version_jar_file: mc_dir.join("versions/1.18.2/1.18.2.jar"),
            version: "1.18.2".to_string(),
            version_type: "release".to_string(),
        };

        let l = builder.launch_instance(settings, None);

        let profile = VersionProfileBuilder::new()
            .id(mock.create_id())
            .instance(l)
            .is_downloaded(true)
            .name("name".into())
            .build();

        mock.add_profile(profile);

        write_toml_config(&mock, "./configs/Profiles.toml")
            .await
            .unwrap();
    }
}

use anyhow::anyhow;
use const_typed_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::instance::launch::LaunchInstance;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VersionProfilesConfig {
    pub profiles: Vec<VersionProfile>,
}

impl VersionProfilesConfig {
    pub fn add_profile(&mut self, profile: VersionProfile) {
        self.profiles.push(profile);
    }

    /// Create an id for the profile
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Loader {
    Vanilla,
    Fabric,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProfileState {
    Downloaded(Box<LaunchInstance>),

    NotDownloaded { version: String, loader: Loader },
}

impl ProfileState {
    pub fn downloaded(instance: LaunchInstance) -> Self {
        Self::Downloaded(Box::new(instance))
    }

    pub fn not_downloaded(version: String, loader: Loader) -> Self {
        Self::NotDownloaded { version, loader }
    }
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct VersionProfile {
    pub id: i32,
    pub name: String,

    pub state: ProfileState,
}

impl VersionProfile {
    pub async fn launch(&self) -> anyhow::Result<()> {
        match &self.state {
            ProfileState::Downloaded(instance) => instance.launch().await,
            ProfileState::NotDownloaded { .. } => Err(anyhow!("This profile is not downloaded!")),
        }
    }

    pub fn version(&self) -> &str {
        match &self.state {
            ProfileState::Downloaded(instance) => &instance.settings.version,
            ProfileState::NotDownloaded { version, .. } => version,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::write_toml_config,
        game_paths::GamePaths,
        instance::{launch::LaunchSettings, InstanceBuilder},
        loaders::fabric::Fabric,
        repository::{java_runner::JavaRunner, username::Username},
    };

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mut mock = VersionProfilesConfig { profiles: vec![] };

        let (tx, _rx) = tokio::sync::mpsc::channel(100);

        let game_paths = GamePaths {
            game: "./minecraft".into(),
            assets: "./minecraft/assets".into(),
            version: "./minecraft/versions/1.20".into(),
            libraries: "./minecraft/libraries".into(),
        };

        let builder = InstanceBuilder::new()
            .version("1.20".into())
            .game_paths(game_paths.clone())
            .instance(Box::new(
                Fabric::new("1.20", None::<String>, game_paths)
                    .await
                    .unwrap(),
            ))
            // .instance(Inner::vanilla("1.20").await.unwrap())
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
            .state(ProfileState::Downloaded(Box::new(l)))
            .name("name".into())
            .build();

        mock.add_profile(profile);

        write_toml_config(&mock, "./configs/Profiles.toml")
            .await
            .unwrap();
    }
}

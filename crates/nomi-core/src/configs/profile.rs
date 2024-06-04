use std::{fmt::Display, sync::Arc};

use anyhow::anyhow;
use const_typed_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    instance::launch::{arguments::UserData, LaunchInstance},
    repository::{java_runner::JavaRunner, manifest::VersionType},
};

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
    pub fn create_id(&self) -> u32 {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Loader {
    Vanilla,
    Fabric { version: Option<String> },
}

impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loader::Vanilla => f.write_str("Vanilla"),
            Loader::Fabric { .. } => f.write_str("Fabric"),
        }
    }
}

impl PartialEq for Loader {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProfileState {
    Downloaded(Arc<LaunchInstance>),

    NotDownloaded {
        version: String,
        version_type: VersionType,
        loader: Loader,
    },
}

impl ProfileState {
    pub fn downloaded(instance: LaunchInstance) -> Self {
        Self::Downloaded(Arc::new(instance))
    }

    pub fn not_downloaded(version: String, version_type: VersionType, loader: Loader) -> Self {
        Self::NotDownloaded {
            version,
            version_type,
            loader,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
pub struct VersionProfile {
    pub id: u32,
    pub name: String,

    pub state: ProfileState,
}

impl VersionProfile {
    pub async fn launch(
        &self,
        user_data: UserData,
        java_runner: &JavaRunner,
    ) -> anyhow::Result<()> {
        match &self.state {
            ProfileState::Downloaded(instance) => instance.launch(user_data, java_runner).await,
            ProfileState::NotDownloaded { .. } => Err(anyhow!("This profile is not downloaded!")),
        }
    }

    pub fn loader_name(&self) -> String {
        match &self.state {
            ProfileState::Downloaded(instance) => instance
                .loader_profile()
                .map_or(format!("{}", Loader::Vanilla), |profile| {
                    format!("{}", profile.loader)
                }),
            ProfileState::NotDownloaded { loader, .. } => format!("{loader}"),
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
        repository::{java_runner::JavaRunner, manifest::VersionType},
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
            assets: mc_dir.join("assets"),
            game_dir: mc_dir.clone(),
            java_bin: JavaRunner::default(),
            libraries_dir: mc_dir.clone().join("libraries"),
            manifest_file: mc_dir.clone().join("versions/1.18.2/1.18.2.json"),
            natives_dir: mc_dir.clone().join("versions/1.18.2/natives"),
            version_jar_file: mc_dir.join("versions/1.18.2/1.18.2.jar"),
            version: "1.18.2".to_string(),
            version_type: VersionType::Release,
        };

        let l = builder.launch_instance(settings, None);

        let profile = VersionProfileBuilder::new()
            .id(mock.create_id())
            .state(ProfileState::downloaded(l))
            .name("name".into())
            .build();

        mock.add_profile(profile);

        write_toml_config(&mock, "./configs/Profiles.toml")
            .await
            .unwrap();
    }
}

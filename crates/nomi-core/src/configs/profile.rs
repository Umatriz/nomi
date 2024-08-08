use std::fmt::Display;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    instance::{
        launch::{arguments::UserData, LaunchInstance},
        logs::GameLogsWriter,
    },
    repository::{java_runner::JavaRunner, manifest::VersionType},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric {
        version: Option<String>,
    },
    Forge,
}

impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loader::Vanilla => f.write_str("Vanilla"),
            Loader::Fabric { .. } => f.write_str("Fabric"),
            Loader::Forge => f.write_str("Forge"),
        }
    }
}

impl Loader {
    pub fn is_fabric(&self) -> bool {
        matches!(*self, Self::Fabric { .. })
    }

    pub fn is_forge(&self) -> bool {
        matches!(*self, Self::Fabric { .. })
    }

    pub fn is_vanilla(&self) -> bool {
        matches!(*self, Self::Vanilla)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProfileState {
    Downloaded(Box<LaunchInstance>),

    NotDownloaded {
        version: String,
        version_type: VersionType,
        loader: Loader,
    },
}

impl ProfileState {
    pub fn downloaded(instance: LaunchInstance) -> Self {
        Self::Downloaded(Box::new(instance))
    }

    pub fn not_downloaded(version: String, version_type: VersionType, loader: Loader) -> Self {
        Self::NotDownloaded {
            version,
            version_type,
            loader,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, TypedBuilder, Clone, PartialEq, Eq, Hash)]
pub struct VersionProfile {
    pub id: usize,
    pub name: String,

    pub state: ProfileState,
}

impl VersionProfile {
    pub async fn launch(&self, user_data: UserData, java_runner: &JavaRunner, logs_writer: &dyn GameLogsWriter) -> anyhow::Result<()> {
        match &self.state {
            ProfileState::Downloaded(instance) => instance.launch(user_data, java_runner, logs_writer).await,
            ProfileState::NotDownloaded { .. } => Err(anyhow!("This profile is not downloaded!")),
        }
    }

    pub fn loader(&self) -> Loader {
        match &self.state {
            ProfileState::Downloaded(instance) => instance.loader_profile().map_or(Loader::Vanilla, |profile| profile.loader.clone()),
            ProfileState::NotDownloaded { loader, .. } => loader.clone(),
        }
    }

    pub fn loader_name(&self) -> String {
        match &self.state {
            ProfileState::Downloaded(instance) => instance
                .loader_profile()
                .map_or(format!("{}", Loader::Vanilla), |profile| format!("{}", profile.loader)),
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

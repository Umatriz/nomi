use std::{fmt::Display, sync::Arc};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    instance::launch::{arguments::UserData, LaunchInstance},
    repository::{java_runner::JavaRunner, manifest::VersionType},
};

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

#[derive(Serialize, Deserialize, Debug, TypedBuilder, Clone)]
pub struct VersionProfile {
    pub id: usize,
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

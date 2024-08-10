pub mod builder_ext;
pub mod launch;
pub mod loader;
pub mod logs;
pub mod marker;
mod profile;

use std::path::{Path, PathBuf};

pub use profile::*;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    configs::profile::{Loader, VersionProfile},
    fs::{read_toml_config_sync, write_toml_config},
    INSTANCES_DIR, INSTANCE_CONFIG,
};

/// Loads all instances in the [`INSTANCES_DIR`](crate::consts::INSTANCES_DIR)
pub fn load_instances() -> anyhow::Result<Vec<Instance>> {
    let dir = std::fs::read_dir(INSTANCES_DIR)?;

    let mut instances = Vec::new();

    for entry in dir {
        let Ok(entry) = entry.inspect_err(|error| error!(%error, "Cannot read instance directory")) else {
            continue;
        };

        if !entry.path().is_dir() {
            continue;
        }

        let path = entry.path().join(INSTANCE_CONFIG);

        let instance = read_toml_config_sync::<Instance>(path)?;

        instances.push(instance);
    }

    Ok(instances)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    name: String,
    id: usize,
    main_profile: Option<InstanceProfileId>,
    profiles: Vec<ProfilePayload>,
}

impl Instance {
    pub fn new(name: impl Into<String>, id: usize) -> Self {
        Self {
            name: name.into(),
            id,
            main_profile: None,
            profiles: Vec::new(),
        }
    }

    pub fn set_main_profile(&mut self, main_profile_id: InstanceProfileId) {
        self.main_profile = Some(main_profile_id);
    }

    pub fn add_profile(&mut self, payload: ProfilePayload) {
        self.profiles.push(payload);
    }

    /// Generate id for the next profile in this instance
    pub fn next_id(&self) -> InstanceProfileId {
        match &self.profiles.iter().max_by_key(|profile| profile.id.1) {
            Some(profile) => InstanceProfileId::new(profile.id.0, profile.id.1 + 1),
            None => InstanceProfileId::new(self.id, 0),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn write(&self) -> anyhow::Result<()> {
        write_toml_config(&self, PathBuf::from(INSTANCES_DIR).join(&self.name).join(".nomi/Instance.toml")).await
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(INSTANCES_DIR).join(&self.name)
    }
}

/// Represent a unique identifier of a profile.
///
/// First number is the instance id and the second number is the profile id.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct InstanceProfileId(usize, usize);

impl InstanceProfileId {
    pub const ZERO: Self = Self(0, 0);

    pub fn new(instance: usize, profile: usize) -> Self {
        Self(instance, profile)
    }
}

/// Information about profile.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfilePayload {
    pub id: InstanceProfileId,
    pub name: String,
    pub loader: Loader,
    pub version: String,
    pub path: PathBuf,
}

impl ProfilePayload {
    pub fn from_version_profile(profile: &VersionProfile, path: &Path) -> Self {
        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            loader: profile.loader().clone(),
            version: profile.version().to_owned(),
            path: path.to_path_buf(),
        }
    }
}

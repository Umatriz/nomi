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
    fs::{read_toml_config_sync, write_toml_config, write_toml_config_sync},
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn main_profile(&self) -> Option<InstanceProfileId> {
        self.main_profile
    }

    pub fn profiles(&self) -> &[ProfilePayload] {
        &self.profiles
    }

    pub fn profiles_mut(&mut self) -> &mut Vec<ProfilePayload> {
        &mut self.profiles
    }

    pub async fn write(&self) -> anyhow::Result<()> {
        write_toml_config(&self, self.path().join(".nomi/Instance.toml")).await
    }

    pub fn write_blocking(&self) -> anyhow::Result<()> {
        write_toml_config_sync(&self, self.path().join(".nomi/Instance.toml"))
    }

    pub fn path(&self) -> PathBuf {
        Self::path_from_id(self.id)
    }

    pub fn path_from_id(id: usize) -> PathBuf {
        PathBuf::from(INSTANCES_DIR).join(format!("{id}"))
    }
}

/// Represent a unique identifier of a profile.
///
/// First number is the instance id and the second number is the profile id.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceProfileId(usize, usize);

impl InstanceProfileId {
    pub const ZERO: Self = Self(0, 0);

    pub fn new(instance: usize, profile: usize) -> Self {
        Self(instance, profile)
    }

    pub fn instance(&self) -> usize {
        self.0
    }

    pub fn profile(&self) -> usize {
        self.1
    }
}

/// Information about profile.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfilePayload {
    pub id: InstanceProfileId,
    pub name: String,
    pub loader: Loader,
    pub version: String,
    pub is_downloaded: bool,
    pub path: PathBuf,
}

impl ProfilePayload {
    pub fn from_version_profile(profile: &VersionProfile, path: &Path) -> Self {
        Self {
            id: profile.id,
            name: profile.name.clone(),
            loader: profile.loader().clone(),
            version: profile.version().to_owned(),
            is_downloaded: profile.is_downloaded(),
            path: path.to_path_buf(),
        }
    }
}

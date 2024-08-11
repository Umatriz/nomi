use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, LazyLock},
};
use tracing::error;

use nomi_core::{fs::read_toml_config_sync, instance::InstanceProfileId};
use parking_lot::RwLock;

use crate::{errors_pool::ErrorPoolExt, views::ModdedProfile};

pub static GLOBAL_CACHE: LazyLock<Arc<RwLock<GlobalCache>>> = LazyLock::new(|| Arc::new(RwLock::new(GlobalCache::new())));

pub struct GlobalCache {
    profiles: HashMap<InstanceProfileId, Arc<RwLock<ModdedProfile>>>,
}

impl GlobalCache {
    pub fn new() -> Self {
        Self { profiles: HashMap::new() }
    }

    pub fn request_profile(&mut self, id: InstanceProfileId, path: PathBuf) -> Option<Arc<RwLock<ModdedProfile>>> {
        match self.profiles.get(&id) {
            Some(some) => Some(some.clone()),
            None => read_toml_config_sync(path)
                .inspect_err(|error| error!(%error, "Cannot read profile config"))
                .report_error()
                .and_then(|profile| {
                    self.profiles.insert(id, profile);
                    self.profiles.get(&id).cloned()
                }),
        }
    }
}

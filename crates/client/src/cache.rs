use eframe::egui::Ui;
use itertools::Itertools;
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

impl Default for GlobalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalCache {
    pub fn new() -> Self {
        Self { profiles: HashMap::new() }
    }

    pub fn get_profile(&self, id: InstanceProfileId) -> Option<Arc<RwLock<ModdedProfile>>> {
        self.profiles.get(&id).cloned()
    }

    pub fn load_profile(&mut self, id: InstanceProfileId, path: PathBuf) -> Option<Arc<RwLock<ModdedProfile>>> {
        read_toml_config_sync(path)
            .inspect_err(|error| error!(%error, "Cannot read profile config"))
            .report_error()
            .and_then(|profile| {
                self.profiles.insert(id, profile);
                self.profiles.get(&id).cloned()
            })
    }

    fn loaded_profiles(&self) -> Vec<Arc<RwLock<ModdedProfile>>> {
        self.profiles.values().cloned().collect_vec()
    }
}

pub fn ui_for_loaded_profiles(ui: &mut Ui) {
    ui.vertical(|ui| {
        for profile in GLOBAL_CACHE.read().loaded_profiles() {
            let profile = profile.read();
            ui.horizontal(|ui| {
                ui.label(&profile.profile.name);
                ui.label(profile.profile.version());
            });
            ui.separator();
        }
    });
}

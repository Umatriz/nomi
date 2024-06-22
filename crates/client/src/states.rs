use std::{collections::HashSet, path::PathBuf};

use nomi_core::{
    fs::read_toml_config_sync, DOT_NOMI_JAVA_EXECUTABLE, DOT_NOMI_PROFILES_CONFIG,
    DOT_NOMI_SETTINGS_CONFIG,
};

use crate::{
    components::{
        add_tab_menu::TabsState,
        profiles::ProfilesState,
        settings::{ClientSettingsState, SettingsState},
        tasks_manager::TasksManagerState,
    },
    errors_pool::ErrorsPoolState,
    TabId,
};

pub struct States {
    pub tabs: TabsState,
    pub errors_pool: ErrorsPoolState,

    pub java: JavaState,
    pub profiles: ProfilesState,
    pub settings: SettingsState,
    pub client_settings: ClientSettingsState,
    pub download_progress: TasksManagerState,
}

impl Default for States {
    fn default() -> Self {
        let mut tabs = HashSet::new();

        tabs.insert(TabId::PROFILES);
        tabs.insert(TabId::LOGS);
        tabs.insert(TabId::SETTINGS);
        tabs.insert(TabId::DOWNLOAD_PROGRESS);

        let settings =
            read_toml_config_sync::<SettingsState>(DOT_NOMI_SETTINGS_CONFIG).unwrap_or_default();

        Self {
            tabs: TabsState(tabs),
            java: JavaState::new(),
            errors_pool: ErrorsPoolState::default(),
            profiles: read_toml_config_sync::<ProfilesState>(DOT_NOMI_PROFILES_CONFIG)
                .unwrap_or_default(),
            client_settings: settings.client_settings.clone(),
            settings,
            download_progress: TasksManagerState::default(),
        }
    }
}

impl States {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct JavaState {
    pub is_downloaded: bool,
}

impl JavaState {
    pub fn new() -> Self {
        let res = std::process::Command::new("java").arg("--version").spawn();
        Self {
            is_downloaded: res.is_ok() || PathBuf::from(DOT_NOMI_JAVA_EXECUTABLE).exists(),
        }
    }
}

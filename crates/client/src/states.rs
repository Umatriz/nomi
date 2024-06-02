use std::collections::HashSet;

use nomi_core::fs::read_toml_config_sync;

use crate::{
    components::{
        add_tab_menu::TabsState,
        download_progress::DownloadProgressState,
        profiles::ProfilesState,
        settings::{ClientSettingsState, SettingsState},
    },
    TabId,
};

pub struct States {
    pub tabs: TabsState,

    pub profiles: ProfilesState,
    pub settings: SettingsState,
    pub client_settings: ClientSettingsState,
    pub download_progress: DownloadProgressState,
}

impl States {
    pub fn new() -> anyhow::Result<Self> {
        let mut tabs = HashSet::new();

        tabs.insert(TabId::PROFILES);
        tabs.insert(TabId::SETTINGS);

        let settings = read_toml_config_sync::<SettingsState>("./.nomi/configs/Settings.toml")
            .unwrap_or_default();

        Ok(Self {
            tabs: TabsState(tabs),
            profiles: read_toml_config_sync::<ProfilesState>("./.nomi/configs/Profiles.toml")
                .unwrap_or_default(),
            client_settings: settings.client_settings.clone(),
            settings,
            download_progress: DownloadProgressState::default(),
        })
    }
}

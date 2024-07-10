use std::sync::Arc;

use nomi_core::configs::profile::VersionProfile;

use crate::views::SimpleProfile;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum TabKind {
    Profiles,
    Mods { profile: SimpleProfile },
    ProfileInfo { profile: Arc<VersionProfile> },
    Settings,
    Logs,
    DownloadProgress,
}

impl TabKind {
    pub const AVAILABLE_TABS_TO_OPEN: &'static [Self] = &[
        Self::Profiles,
        Self::Settings,
        Self::Logs,
        Self::DownloadProgress,
    ];

    pub fn name(&self) -> String {
        match self {
            TabKind::Profiles => "Profiles".to_owned(),
            TabKind::Mods { profile } => format!(
                "Mods ({}, {}, {})",
                profile.name, profile.version, profile.loader
            ),
            TabKind::ProfileInfo { profile } => format!("Profile ({})", profile.name),
            TabKind::Settings => "Settings".to_owned(),
            TabKind::Logs => "Logs".to_owned(),
            TabKind::DownloadProgress => "Progress".to_owned(),
        }
    }
}

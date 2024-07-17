use std::{ops::Deref, sync::Arc};

use parking_lot::RwLock;

use crate::views::ModdedProfile;

pub struct Tab {
    pub id: TabId,
    pub kind: TabKind,
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TabId(String);

impl Deref for TabId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Clone)]
pub enum TabKind {
    Profiles,
    Mods { profile: Arc<RwLock<ModdedProfile>> },
    ProfileInfo { profile: Arc<RwLock<ModdedProfile>> },
    Settings,
    Logs,
    DownloadProgress,
}

impl TabKind {
    pub const AVAILABLE_TABS_TO_OPEN: &'static [Self] = &[Self::Profiles, Self::Settings, Self::Logs, Self::DownloadProgress];

    #[doc(alias = "name")]
    pub fn id(&self) -> TabId {
        let id = match self {
            TabKind::Profiles => "Profiles".to_owned(),
            TabKind::Mods { profile } => {
                let profile = profile.read();
                format!(
                    "Mods ({}, {}, {})",
                    profile.profile.name,
                    profile.profile.version(),
                    profile.profile.loader_name()
                )
            }
            TabKind::ProfileInfo { profile } => {
                let profile = profile.read();
                format!("Profile ({})", profile.profile.name)
            }
            TabKind::Settings => "Settings".to_owned(),
            TabKind::Logs => "Logs".to_owned(),
            TabKind::DownloadProgress => "Progress".to_owned(),
        };

        TabId(id)
    }
}

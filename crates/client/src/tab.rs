#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TabId(&'static str);

impl TabId {
    pub const MODS: Self = Self("Mods");
    pub const PROFILES: Self = Self("Profiles");
    pub const SETTINGS: Self = Self("Settings");
    pub const LOGS: Self = Self("Logs");
    pub const DOWNLOAD_PROGRESS: Self = Self("Download Progress");
}

#[derive(PartialEq)]
pub struct Tab {
    id: TabId,
    kind: TabKind,
}

impl Tab {
    pub fn from_tab_kind(kind: TabKind) -> Self {
        Self {
            id: kind.id(),
            kind,
        }
    }

    pub fn id(&self) -> &TabId {
        &self.id
    }

    pub fn kind(&self) -> &TabKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut TabKind {
        &mut self.kind
    }
}

pub enum TabKind {
    Profiles,
    Mods,
    Settings,
    Logs,
    DownloadProgress,
}

impl PartialEq for TabKind {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl TabKind {
    pub const AVAILABLE_TABS_TO_OPEN: &'static [Self] = &[
        Self::Profiles,
        Self::Mods,
        Self::Settings,
        Self::Logs,
        Self::DownloadProgress,
    ];

    pub fn from_id(id: TabId) -> Self {
        match id {
            TabId::PROFILES => TabKind::Profiles,
            TabId::SETTINGS => TabKind::Settings,
            TabId::LOGS => TabKind::Logs,
            TabId::DOWNLOAD_PROGRESS => TabKind::DownloadProgress,
            TabId::MODS => TabKind::Mods,
            _ => unreachable!(),
        }
    }

    pub fn id(&self) -> TabId {
        match self {
            TabKind::Profiles => TabId::PROFILES,
            TabKind::Settings => TabId::SETTINGS,
            TabKind::Logs => TabId::LOGS,
            TabKind::DownloadProgress => TabId::DOWNLOAD_PROGRESS,
            TabKind::Mods => TabId::MODS,
        }
    }

    pub fn name(&self) -> &'static str {
        self.id().0
    }
}

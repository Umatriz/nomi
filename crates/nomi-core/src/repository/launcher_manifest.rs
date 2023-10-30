use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LauncherManifest {
    pub latest: LauncherManifestLatest,
    pub versions: Vec<LauncherManifestVersion>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LauncherManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct LauncherManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

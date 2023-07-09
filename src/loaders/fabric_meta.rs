use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Meta(pub Vec<VersionLoader>);

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionLoader {
    pub loader: Loader,
    pub intermediary: Intermediary,
    pub launcher_meta: LauncherMeta,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Loader {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Intermediary {
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LauncherMeta {
    pub version: i32,
    pub libraries: Libraries,
    pub main_class: MainClass,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Libraries {
    pub client: Vec<Library>,
    pub common: Vec<Library>,
    pub server: Vec<Library>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MainClass {
    pub client: String,
    pub server: String,
}

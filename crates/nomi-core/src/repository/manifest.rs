use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    pub asset_index: ManifestAssetIndex,
    pub assets: String,
    pub compliance_level: i8,
    pub downloads: ManifestDownloads,
    pub id: String,
    pub java_version: ManifestJavaVersion,
    pub libraries: Vec<ManifestLibrary>,
    pub main_class: String,
    pub minimum_launcher_version: i8,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub version_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Arguments {
    New {
        game: Vec<Argument>,
        jvm: Vec<Argument>,
    },
    Old(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Argument {
    Struct { rules: Vec<Rule>, value: Value },
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule {
    pub action: Action,
    #[serde(flatten)]
    pub rule_kind: Option<RuleKind>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RuleKind {
    #[serde(rename = "features")]
    GameRule(Features),
    #[serde(rename = "os")]
    JvmRule(Os),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Allow,
    Disallow,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Features {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Os {
    pub arch: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestAssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i32,
    pub total_size: i32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManifestDownloads {
    pub client: ManifestFile,
    pub client_mappings: Option<ManifestFile>,
    pub server: Option<ManifestFile>,
    pub server_mappings: Option<ManifestFile>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub path: Option<String>,
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestJavaVersion {
    pub component: String,
    pub major_version: i8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestLibrary {
    pub downloads: ManifestLibraryDownloads,
    pub name: String,
    // pub natives: Option<ManifestNatives>,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestLibraryDownloads {
    pub artifact: Option<ManifestFile>,
    pub classifiers: Option<ManifestClassifiers>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestClassifiers {
    pub natives_macos: Option<ManifestFile>,
    pub natives_windows: Option<ManifestFile>,
    pub natives_linux: Option<ManifestFile>,
}

#[cfg(test)]
mod tests {

    use reqwest::get;

    use super::*;

    #[tokio::test]
    async fn old_version_test() {
        let manifest: Manifest = get("https://piston-meta.mojang.com/v1/packages/d546f1707a3f2b7d034eece5ea2e311eda875787/1.8.9.json").await.unwrap().json().await.unwrap();
        println!("{:#?}", manifest.arguments)
    }

    #[tokio::test]
    async fn deserialize_test() {
        let manifest: Manifest = get("https://piston-meta.mojang.com/v1/packages/334b33fcba3c9be4b7514624c965256535bd7eba/1.18.2.json").await.unwrap().json().await.unwrap();
        println!("{:#?}", manifest.libraries[29])
    }
}

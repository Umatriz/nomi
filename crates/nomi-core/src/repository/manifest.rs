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
        game: Vec<JvmArgument>,
        jvm: Vec<JvmArgument>,
    },
    Old(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JvmArgument {
    String(String),
    Struct {
        rules: Vec<Rules>,
        value: serde_json::Value,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rules {
    pub action: String,
    pub features: Option<Features>,
    pub os: Option<Os>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Features {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub server: ManifestFile,
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
    pub rules: Option<Vec<Rules>>,
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
    use crate::utils::get;

    use super::*;

    #[tokio::test]
    async fn old_version_test() {
        let manifest: Manifest = get("https://piston-meta.mojang.com/v1/packages/832d95b9f40699d4961394dcf6cf549e65f15dc5/1.12.2.json").await.unwrap();
        println!("{:#?}", manifest)
    }
}

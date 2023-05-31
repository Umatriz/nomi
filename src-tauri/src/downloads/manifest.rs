use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Manifest {
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
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
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ManifestFile {
  pub path: Option<String>,
  pub sha1: String,
  pub size: i32,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ManifestJavaVersion {
  pub component: String,
  pub major_version: i8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ManifestLibrary {
  pub downloads: ManifestLibraryDownloads,
  pub name: String,
  pub rules: Option<Vec<ManifestRule>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ManifestLibraryDownloads {
  pub artifact: Option<ManifestFile>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ManifestRule {
  pub action: String,
  pub os: Option<HashMap<String, String>>,
}
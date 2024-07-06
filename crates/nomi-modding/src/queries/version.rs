//! Version

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub version_type: String,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub status: String,
    pub requested_status: String,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub date_published: String,
    pub downloads: i64,
    pub changelog_url: Option<serde_json::Value>,
    pub files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
pub struct Dependency {
    pub version_id: String,
    pub project_id: String,
    pub file_name: String,
    pub dependency_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

use const_typed_builder::Builder;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Builder)]
pub struct VersionProfile {
    pub id: i32,
    pub is_downloaded: bool,

    pub version: String,
    pub version_type: String,
    pub version_jar_file: PathBuf,

    pub assets: PathBuf,
    pub game_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub natives_dir: PathBuf,
}

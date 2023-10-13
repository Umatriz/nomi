use std::path::{Path, PathBuf};

pub struct LaunchInstance {}

pub struct LaunchInstanceBuilder {
    pub access_token: Option<String>,
    pub username: String,
    pub uuid: Option<String>,

    pub assets: PathBuf,
    pub game_dir: PathBuf,
    pub java_bin: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub natives_dir: PathBuf,
    pub version_jar_file: PathBuf,

    pub version: String,
    pub version_type: String,
}

impl LaunchInstanceBuilder {
    pub fn access_token(&mut self, access_token: Option<String>) {
        self.access_token = access_token;
    }

    pub fn username(&mut self, username: impl Into<String>) {
        self.username = username.into();
    }

    pub fn uuid(&mut self, uuid: Option<String>) {
        self.uuid = uuid;
    }

    pub fn assets(&mut self, assets: impl AsRef<Path>) {
        self.assets = assets.as_ref().to_path_buf();
    }

    pub fn game_dir(&mut self, game_dir: impl AsRef<Path>) {
        self.game_dir = game_dir.as_ref().to_path_buf();
    }

    pub fn java_bin(&mut self, java_bin: impl AsRef<Path>) {
        self.java_bin = java_bin.as_ref().to_path_buf();
    }

    pub fn libraries_dir(&mut self, libraries_dir: impl AsRef<Path>) {
        self.libraries_dir = libraries_dir.as_ref().to_path_buf();
    }

    pub fn manifest_file(&mut self, manifest_file: impl AsRef<Path>) {
        self.manifest_file = manifest_file.as_ref().to_path_buf();
    }

    pub fn natives_dir(&mut self, natives_dir: impl AsRef<Path>) {
        self.natives_dir = natives_dir.as_ref().to_path_buf();
    }

    pub fn version_jar_file(&mut self, version_jar_file: impl AsRef<Path>) {
        self.version_jar_file = version_jar_file.as_ref().to_path_buf();
    }

    pub fn version(&mut self, version: impl Into<String>) {
        self.version = version.into();
    }

    pub fn version_type(&mut self, version_type: impl Into<String>) {
        self.version_type = version_type.into();
    }

    // TODO: Add `build`
}

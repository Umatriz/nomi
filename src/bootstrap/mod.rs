mod classpath;

use std::{path::{Path}, process::Command};

use thiserror::Error;

use crate::manifest::read_manifest_from_file;

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("The game directory doesn't exist.")]
    GameDirNotExist,

    #[error("The java bin doesn't exist.")]
    JavaBinNotExist,

    #[error("The version file (.json) doesn't exist.")]
    VersionFileNotFound,

    #[error("An unexpected error has ocurred.")]
    UnknownError,

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct Version {
  version: String,
  version_type: String,
  // max_ram: i8,
  // min_ram: i8,
  username: String,
  // password: Option<String>,
  uuid: String,
  access_token: String,
  dir: String,
  java_bin: String,
}


// TODO: MAX/MIN ram
impl Version {
  pub fn new(
    version: &str,
    version_type: String,
    // max_ram: i8,
    // min_ram: i8,
    username: &str,
    // password: Option<String>,
    access_token: &str,
    dir: &str,
    java_bin: &str,
  ) -> Self {
      Self {
        version: version.to_string(),
        version_type: version_type,
        // max_ram,
        // min_ram,
        username: username.to_string(),
        // password: password,
        uuid: uuid::Uuid::new_v4().to_string(),
        access_token: access_token.to_string(),
        dir: dir.to_string(),
        java_bin: java_bin.to_string(),
    }
  }

  pub fn get_assets_dir(&self) -> String {
    return Path::new(&self.dir)
      .join("assets")
      .to_str()
      .unwrap()
      .to_string();
  }

  pub fn get_libs_dir(&self) -> String {
    return Path::new(&self.dir)
      .join("libraries")
      .to_str()
      .unwrap()
      .to_string();
  }

  pub fn get_json_file(&self) -> String {
    return Path::new(&self.dir)
      .join("versions")
      .join(&self.version)
      .join(format!("{}.json", self.version))
      .to_str()
      .unwrap()
      .to_string();
  }

  pub fn get_jar_file(&self) -> String {
    return Path::new(&self.dir)
      .join("versions")
      .join(&self.version)
      .join(format!("{}.jar", self.version))
      .to_str()
      .unwrap()
      .to_string();
  }

  pub fn get_natives_dir(&self) -> String {
    return Path::new(&self.dir)
      .join("versions")
      .join(&self.version)
      .join("natives")
      .to_str()
      .unwrap()
      .to_string();
  }

  pub fn build_args(&self) -> Result<Vec<String>, BootstrapError> {
    if !Path::new(&self.dir).is_dir() {
        return Err(BootstrapError::GameDirNotExist);
    }

    if !Path::new(&self.java_bin).is_file() {
        return Err(BootstrapError::JavaBinNotExist);
    }

    let manifest_file = &self.get_json_file();
    if !Path::new(manifest_file).is_file() {
        return Err(BootstrapError::VersionFileNotFound);
    }

    let manifest = read_manifest_from_file(manifest_file).unwrap();
    let classpath = classpath::create_classpath(
        self.get_jar_file(),
        self.get_libs_dir(),
        manifest.libraries,
    );

    let args: Vec<String> = vec![
        format!("-Djava.library.path={}", self.get_natives_dir()),
        format!("-Dminecraft.launcher.brand={}", "nomi"),
        format!("-Dminecraft.launcher.version={}", "0.0.01"),
        format!("-cp"),
        format!("{}", classpath),
        format!("{}", manifest.main_class),
        format!("--accessToken"),
        format!("{}", self.access_token),
        format!("--assetsDir"),
        format!("{}", self.get_assets_dir()),
        format!("--assetIndex"),
        format!("{}", manifest.asset_index.id),
        format!("--gameDir"),
        format!("{}", self.dir),
        format!("--userType"),
        format!("{}", "mojang"),
        format!("--username"),
        format!("{}", self.username),
        format!("--uuid"),
        format!("{}", self.uuid),
        format!("--version"),
        format!("{}", self.version),
        format!("--versionType"),
        format!("{}", self.version_type),
    ];

    return Ok(args);
  }

  pub fn launch(&self) -> Result<i32, BootstrapError> {
    let args = self.build_args().unwrap();

    let mut process = Command::new(&self.java_bin)
      .args(args)
      .spawn()
      .expect("command failed to start");

    let status = process.wait().unwrap().code().unwrap();
    return Ok(status);
}
}
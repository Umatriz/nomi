mod classpath;
pub mod rules;

use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result};
use thiserror::Error;

use crate::repository::manifest::{read_manifest_from_file, JvmArgument};

use self::rules::is_all_rules_satisfied;

#[cfg(windows)]
const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(windows))]
const CLASSPATH_SEPARATOR: &str = ":";

// TODO: IMPORTANT Rewrite all this

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

#[derive(Default)]
pub struct ClientSettings {
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

pub struct ClientBootstrap {
    pub settings: ClientSettings,
}

impl ClientBootstrap {
    pub fn new(settings: ClientSettings) -> Self {
        Self { settings }
    }

    pub fn get_assets_dir(&self) -> PathBuf {
        self.settings.assets.clone()
    }

    pub fn get_game_dir(&self) -> PathBuf {
        self.settings.game_dir.clone()
    }

    pub fn get_json_file(&self) -> PathBuf {
        self.settings.manifest_file.clone()
    }

    pub fn get_jar_file(&self) -> PathBuf {
        self.settings.version_jar_file.clone()
    }

    pub fn get_libs_dir(&self) -> PathBuf {
        self.settings.libraries_dir.clone()
    }

    pub fn get_natives_dir(&self) -> PathBuf {
        self.settings.natives_dir.clone()
    }

    pub fn build_args(&self) -> Result<Vec<String>> {
        let assets_dir = self.get_assets_dir();
        let game_dir = self.get_game_dir();
        let java_bin = self.settings.java_bin.clone();
        let json_file = self.get_json_file();
        let natives_dir = self.get_natives_dir();

        if !game_dir.is_dir() {
            return Err(BootstrapError::GameDirNotExist.into());
        }

        if !java_bin.is_file() {
            return Err(BootstrapError::JavaBinNotExist.into());
        }

        if !json_file.is_file() {
            return Err(BootstrapError::VersionFileNotFound.into());
        }

        let manifest = read_manifest_from_file(&json_file)?;

        let assets_index = &manifest.asset_index.id;
        let classpath = classpath::create_classpath(
            self.settings.version_jar_file.clone(),
            self.settings.libraries_dir.clone(),
            manifest.libraries,
        )?;

        let mut args: Vec<String> = vec![];

        for arg in manifest.arguments.jvm {
            match arg {
                JvmArgument::String(value) => {
                    args.push(value);
                }
                JvmArgument::Struct { value, rules, .. } => {
                    if !is_all_rules_satisfied(&rules)? {
                        continue;
                    }

                    if let Some(value) = value.as_str() {
                        args.push(value.to_string());
                    } else if let Some(value_arr) = value.as_array() {
                        for value in value_arr {
                            if let Some(value) = value.as_str() {
                                args.push(value.to_string());
                            }
                        }
                    }
                }
            }
        }

        args.push(manifest.main_class);

        for arg in manifest.arguments.game {
            match arg {
                JvmArgument::String(value) => {
                    args.push(value);
                }
                JvmArgument::Struct { value, rules, .. } => {
                    if !is_all_rules_satisfied(&rules)? {
                        continue;
                    }

                    if let Some(value) = value.as_str() {
                        args.push(value.to_string());
                    } else if let Some(value_arr) = value.as_array() {
                        for value in value_arr {
                            if let Some(value) = value.as_str() {
                                args.push(value.to_string());
                            }
                        }
                    }
                }
            }
        }

        args = args
            .iter()
            .map(|x| {
                // TODO: remove unwraps here
                x.replace("${assets_root}", assets_dir.to_str().unwrap())
                    .replace("${game_directory}", game_dir.to_str().unwrap())
                    .replace("${natives_directory}", natives_dir.to_str().unwrap())
                    .replace("${launcher_name}", "nomi")
                    .replace("${launcher_version}", "0.0.1")
                    .replace(
                        "${auth_access_token}",
                        self.settings
                            .access_token
                            .clone()
                            .unwrap_or("null".to_string())
                            .as_str(),
                    )
                    .replace("${auth_player_name}", self.settings.username.as_str())
                    .replace(
                        "${auth_uuid}",
                        self.settings
                            .uuid
                            .clone()
                            .unwrap_or("null".to_string())
                            .as_str(),
                    )
                    .replace("${version_type}", &self.settings.version_type)
                    .replace("${version_name}", &self.settings.version)
                    .replace("${assets_index_name}", assets_index)
                    .replace("${user_properties}", "{}")
                    .replace("${classpath}", &classpath)
            })
            .collect();

        Ok(args)
    }

    pub fn launch(&self) -> Result<i32> {
        let args = self.build_args()?;

        let mut process =
            // E:\\programming\\code\\nomi\\nomi_core\\java\\jdk-17.0.8\\bin\\java.exe
            Command::new("java")
                // .arg("-Xms256m")
                // .arg("-Xmx1024m")
                .args(args)
                .current_dir(&self.settings.game_dir)
                .spawn()
                .context("command failed to start")?;

        let status = process
            .wait()?
            .code()
            .context("can't get minecraft exit code")?;

        Ok(status)
    }
}

pub fn java_bin() -> Option<PathBuf> {
    let _path = std::env::var("Path").unwrap();
    let path_vec = _path.split(';').collect::<Vec<&str>>();
    let mut java_bin: Option<PathBuf> = None;
    for i in path_vec.iter() {
        if i.contains("java") {
            let pb = PathBuf::from(i).join("java.exe");
            match pb.exists() {
                true => java_bin = Some(pb),
                false => java_bin = None,
            }
        }
    }
    java_bin
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_config() -> ClientSettings {
        let mc_dir = std::env::current_dir().unwrap().join("minecraft");
        ClientSettings {
            assets: mc_dir.join("assets"),
            access_token: None,
            username: "ItWorks".to_string(),
            uuid: None,
            game_dir: mc_dir.clone(),
            java_bin: java_bin().unwrap(),
            libraries_dir: mc_dir.clone().join("libraries"),
            manifest_file: mc_dir
                .clone()
                .join("versions")
                .join("1.18.2")
                .join("1.18.2.json"),
            natives_dir: mc_dir
                .clone()
                .join("versions")
                .join("1.18.2")
                .join("natives"),
            version: "1.18.2".to_string(),
            version_type: "release".to_string(),
            version_jar_file: mc_dir.join("versions").join("1.18.2").join("1.18.2.jar"),
        }
    }

    #[test]
    fn it_works() {
        let settings = fake_config();

        let bootstrap = ClientBootstrap::new(settings);
        bootstrap.launch().unwrap();
    }

    #[test]
    fn args_test() {
        let settings = fake_config();

        let bootstrap = ClientBootstrap::new(settings);
        let args = bootstrap.build_args().unwrap();
        println!("{:?}", args);
    }

    #[test]
    fn java_call() {
        let mut cmd =
            Command::new("E:\\programming\\code\\nomi\\nomi_core\\java\\jdk-17.0.8\\bin\\java.exe")
                .arg("--version")
                .spawn()
                .unwrap();

        cmd.wait().unwrap().code().unwrap();
    }
}

mod classpath;
pub mod profile;
pub mod rules;

use std::{path::PathBuf, process::Command};

use thiserror::Error;

use crate::{
    loaders::{fabric::fabric_meta::FabricProfile, quilt::quilt_meta::QuiltProfile},
    manifest::{read_manifest_from_file, JvmArgument},
};

use self::{profile::Loader, rules::is_all_rules_satisfied};

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

pub struct ClientAuth {
    pub access_token: Option<String>,
    pub username: String,
    pub uuid: Option<String>,
}

pub struct ClientVersion {
    pub version: String,
    pub version_type: VersionType,
    pub loader: Loader,
}

pub struct ClientSettings {
    pub assets: PathBuf,
    pub auth: ClientAuth,
    pub game_dir: PathBuf,
    pub java_bin: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub profile_path: Option<PathBuf>,
    pub natives_dir: PathBuf,
    pub version: ClientVersion,
    pub version_jar_file: PathBuf,
}

pub struct ClientBootstrap {
    pub settings: ClientSettings,
}

pub enum VersionType {
    Release,
    Snapshot,
}

impl ClientBootstrap {
    pub fn new(settings: ClientSettings) -> Self {
        Self { settings }
    }

    pub fn build_args(&self) -> anyhow::Result<Vec<String>> {
        let auth = &self.settings.auth;
        let assets_dir = &self.settings.assets;
        let game_dir = &self.settings.game_dir;
        let java_bin = self.settings.java_bin.clone();
        let json_file = self.settings.manifest_file.clone();
        let natives_dir = &self.settings.natives_dir;
        let version = &self.settings.version;

        if !game_dir.is_dir() {
            return Err(BootstrapError::GameDirNotExist.into());
        }

        if !java_bin.is_file() {
            return Err(BootstrapError::JavaBinNotExist.into());
        }

        if !json_file.is_file() {
            return Err(BootstrapError::VersionFileNotFound.into());
        }

        let manifest = read_manifest_from_file(json_file).unwrap();

        let loader = self
            .settings
            .version
            .loader
            // FIXME: `profile_path`
            .load_profile(self.settings.profile_path.clone().unwrap())?;

        // TODO: replce second unwrap
        let profile_option = loader.unwrap();

        let profile_libs = if let Some(profile) = profile_option {
            profile.get_libraries()
        } else {
            vec![]
        };

        let assets_index = &manifest.asset_index.id;
        let classpath = classpath::create_classpath(
            self.settings.version_jar_file.clone(),
            self.settings.libraries_dir.clone(),
            manifest.libraries,
            profile_libs,
        );

        let mut args: Vec<String> = vec![];

        for arg in manifest.arguments.jvm {
            match arg {
                JvmArgument::String(value) => {
                    args.push(value);
                }
                JvmArgument::Struct { value, rules, .. } => {
                    if !is_all_rules_satisfied(&rules) {
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

        if let Some(profile) = profile_option {
            for i in profile.get_args().jvm.unwrap_or_default() {
                args.push(i)
            }
        }

        args.push(match profile_option {
            Some(profile) => profile.get_main_class(),
            None => manifest.main_class,
        });

        for arg in manifest.arguments.game {
            match arg {
                JvmArgument::String(value) => {
                    args.push(value);
                }
                JvmArgument::Struct { .. } => {
                    break;
                }
            }
        }

        if let Some(profile) = profile_option {
            for i in profile.get_args().game.unwrap_or_default() {
                args.push(i)
            }
        }

        args = args
            .iter()
            .map(|x| {
                x.replace("${assets_root}", assets_dir.to_str().unwrap())
                    .replace("${game_directory}", game_dir.to_str().unwrap())
                    .replace("${natives_directory}", natives_dir.to_str().unwrap())
                    .replace("${launcher_name}", "nomi")
                    .replace("${launcher_version}", "0.0.1")
                    .replace(
                        "${auth_access_token}",
                        auth.access_token
                            .clone()
                            .unwrap_or("null".to_string())
                            .as_str(),
                    )
                    .replace("${auth_player_name}", auth.username.as_str())
                    .replace(
                        "${auth_uuid}",
                        auth.uuid.clone().unwrap_or("null".to_string()).as_str(),
                    )
                    .replace("${version_type}", "release")
                    .replace("${version_name}", &version.version)
                    .replace("${assets_index_name}", assets_index)
                    .replace("${user_properties}", "{}")
                    .replace("${classpath}", &classpath)
            })
            .collect();

        Ok(args)
    }

    pub fn launch(&self) -> Result<i32, BootstrapError> {
        let args = self.build_args().unwrap();

        let mut process = Command::new(&self.settings.java_bin)
            .args(args)
            .spawn()
            .expect("command failed to start");

        let status = process.wait().unwrap().code().unwrap();
        // TODO!: return result instead of 🤮🤮 exit code
        Ok(status)
    }
}

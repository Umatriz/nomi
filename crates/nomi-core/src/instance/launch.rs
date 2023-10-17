use std::path::PathBuf;

use anyhow::Context;
use const_typed_builder::Builder;
use thiserror::Error;
use tokio::process::Command;

use crate::repository::{
    manifest::{read_manifest_from_file, JvmArgument},
    username::Username,
};
use rules::is_all_rules_satisfied;

use self::classpath::classpath;

use super::{profile::Profile, Undefined};

pub mod classpath;
pub mod rules;

#[cfg(windows)]
const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(windows))]
const CLASSPATH_SEPARATOR: &str = ":";

#[derive(Error, Debug)]
pub enum LaunchError {
    #[error("The game directory doesn't exist.")]
    GameDirNotExist,

    #[error("The java bin doesn't exist.")]
    JavaBinNotExist,

    #[error("The version file (.json) doesn't exist.")]
    VersionFileNotFound,
}

#[derive(Default, Builder, PartialEq, Debug)]
pub struct LaunchSettings {
    pub access_token: Option<String>,
    pub username: Username,
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

pub struct LaunchInstance<'a> {
    settings: LaunchSettings,
    profile: Option<&'a dyn Profile>,
}

impl LaunchInstance<'_> {
    fn build_args(self) -> anyhow::Result<Vec<String>> {
        let assets_dir = self.settings.assets.clone();
        let game_dir = self.settings.game_dir.clone();
        let java_bin = self.settings.java_bin.clone();
        let json_file = self.settings.manifest_file.clone();
        let natives_dir = self.settings.natives_dir.clone();

        if !game_dir.is_dir() {
            return Err(LaunchError::GameDirNotExist.into());
        }

        if !java_bin.is_file() {
            return Err(LaunchError::JavaBinNotExist.into());
        }

        if !json_file.is_file() {
            return Err(LaunchError::VersionFileNotFound.into());
        }

        let manifest = read_manifest_from_file(&json_file)?;

        let assets_index = &manifest.asset_index.id;

        let mut args: Vec<String> = vec![];

        if let Some(prof) = self.profile {
            let arguments = prof.arguments();
            arguments.jvm.iter().for_each(|a| {
                dbg!(&a);
                args.push(a.to_owned());
            })
        }

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

        let main_class = if let Some(prof) = self.profile {
            prof.main_class()
        } else {
            manifest.main_class
        };

        args.push(main_class.to_owned());

        for arg in manifest.arguments.game {
            match arg {
                JvmArgument::String(value) => {
                    args.push(value);
                }
                _ => break,
            }
        }

        if let Some(prof) = self.profile {
            let arguments = prof.arguments();
            arguments.game.iter().for_each(|a| {
                dbg!(&a);
                args.push(a.to_owned());
            })
        }

        let extra_libraries = self.profile.map(|prof| prof.libraries());
        let classpath = classpath(
            self.settings.version_jar_file.clone(),
            self.settings.libraries_dir.clone(),
            manifest.libraries,
            extra_libraries,
        )?;

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
                    .replace("${auth_player_name}", self.settings.username.get())
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

    pub async fn launch(self) -> anyhow::Result<i32> {
        let game_dir = self.settings.game_dir.clone();
        let args = self.build_args()?;

        let mut process = Command::new("java")
            .args(args)
            .current_dir(game_dir)
            .spawn()
            .context("command failed to start")?;

        let status = process
            .wait()
            .await?
            .code()
            .context("can't get minecraft exit code")?;

        Ok(status)
    }
}

#[derive(Default)]
pub struct LaunchInstanceBuilder<'a, S> {
    settings: S,
    profile: Option<&'a dyn Profile>,
}

impl LaunchInstanceBuilder<'_, Undefined> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> LaunchInstanceBuilder<'a, Undefined> {
    pub fn settings(self, settings: LaunchSettings) -> LaunchInstanceBuilder<'a, LaunchSettings> {
        LaunchInstanceBuilder {
            settings,
            profile: self.profile,
        }
    }
}

impl<'a, S> LaunchInstanceBuilder<'a, S> {
    pub fn profile<P: Profile>(self, profile: &'a P) -> LaunchInstanceBuilder<'a, S> {
        LaunchInstanceBuilder {
            settings: self.settings,
            profile: Some(profile),
        }
    }
}

impl<'a> LaunchInstanceBuilder<'a, LaunchSettings> {
    pub fn build(self) -> LaunchInstance<'a> {
        LaunchInstance {
            settings: self.settings,
            profile: self.profile,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{instance::profile::read, repository::fabric_profile::FabricProfile};

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let mc_dir = std::env::current_dir().unwrap().join("minecraft");
        let settings = LaunchSettingsBuilder::new()
            .access_token(None)
            .assets(mc_dir.join("assets"))
            .game_dir(mc_dir.clone())
            .java_bin("./java/jdk-17.0.8/bin/java.exe".into())
            .libraries_dir(mc_dir.clone().join("libraries"))
            .manifest_file(
                mc_dir
                    .clone()
                    .join("instances")
                    .join("1.18.2")
                    .join("1.18.2.json"),
            )
            .natives_dir(
                mc_dir
                    .clone()
                    .join("instances")
                    .join("1.18.2")
                    .join("natives"),
            )
            .username(Username::new("ItWorks").unwrap())
            .uuid(None)
            .version("1.18.2".to_string())
            .version_jar_file(mc_dir.join("instances").join("1.18.2").join("1.18.2.jar"))
            .version_type("release".to_string())
            .build();

        let fabric =
            read::<FabricProfile>("./minecraft/instances/1.18.2/fabric-loader-0.14.23-1.18.2.json")
                .await
                .unwrap();

        let builder = LaunchInstanceBuilder::new()
            .settings(settings)
            .profile(&fabric)
            .build();

        builder.launch().await.unwrap();
    }
}

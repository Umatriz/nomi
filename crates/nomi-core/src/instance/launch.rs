use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

use crate::repository::{
    java_runner::JavaRunner,
    manifest::{read_manifest_from_file, Arguments, JvmArgument},
    username::Username,
};
use rules::is_all_rules_satisfied;

use self::classpath::classpath;

use super::{profile::LoaderProfile, Undefined};

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

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct LaunchSettings {
    #[serde(skip)]
    pub access_token: Option<String>,
    #[serde(skip)]
    pub username: Username,
    #[serde(skip)]
    pub uuid: Option<String>,

    pub assets: PathBuf,
    pub java_bin: JavaRunner,
    pub game_dir: PathBuf,
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LaunchInstance {
    settings: LaunchSettings,
    profile: Option<LoaderProfile>,
}

impl LaunchInstance {
    pub fn set_username(&mut self, username: Username) {
        self.settings.username = username
    }

    pub fn set_access_token(&mut self, access_token: Option<String>) {
        self.settings.access_token = access_token
    }

    pub fn set_uuid(&mut self, uuid: Option<String>) {
        self.settings.uuid = uuid
    }

    fn build_args(self) -> anyhow::Result<(Vec<String>, String)> {
        let assets_dir = self.settings.assets.clone();
        let game_dir = self.settings.game_dir.clone();
        let java_bin = self.settings.java_bin.clone();
        let json_file = self.settings.manifest_file.clone();
        let natives_dir = self.settings.natives_dir.clone();

        if !game_dir.is_dir() {
            return Err(LaunchError::GameDirNotExist.into());
        }

        if let JavaRunner::Path(p) = java_bin {
            if !p.is_file() {
                return Err(LaunchError::JavaBinNotExist.into());
            }
        }

        if !json_file.is_file() {
            return Err(LaunchError::VersionFileNotFound.into());
        }

        let manifest = read_manifest_from_file(&json_file)?;

        let assets_index = &manifest.asset_index.id;

        let mut args: Vec<String> = vec![];

        if let Some(prof) = self.profile.as_ref() {
            prof.args.jvm.iter().for_each(|a| {
                dbg!(&a);
                args.push(a.to_owned());
            })
        }

        let extra_libraries = self.profile.as_ref().map(|p| &p.libraries);
        let classpath = classpath(
            Some(self.settings.version_jar_file.clone()),
            self.settings.libraries_dir.clone(),
            manifest.libraries,
            extra_libraries,
        )?;

        if let Arguments::New { ref jvm, .. } = manifest.arguments {
            for arg in jvm {
                match arg {
                    JvmArgument::String(value) => {
                        args.push(value.to_string());
                    }
                    JvmArgument::Struct { value, rules, .. } => {
                        if !is_all_rules_satisfied(rules)? {
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
        }

        if let Arguments::Old(_) = manifest.arguments {
            // args.push("-Xms1024M".into());
            // args.push("-Xmx1024M".into());
            // args.push("-Xss1M".into());
            // args.push("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into());
            // args.push("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into());
            args.push("-Djava.library.path=${natives_directory}".into());
            // args.push("-Dminecraft.launcher.brand=${launcher_name}".into());
            // args.push("-Dminecraft.launcher.version=${launcher_version}".into());
            args.push(format!(
                "-Dminecraft.client.jar={}",
                &self.settings.version_jar_file.display()
            ));
            // args.push("-cp \"${classpath}\"".into());
            // args.push("-jar E:\\programming\\code\\nomi\\crates\\cli\\minecraft\\minecraft\\versions\\1.12.2\\1.12.2.jar".into())
        }

        let main_class = if let Some(ref prof) = self.profile {
            &prof.main_class
        } else {
            &manifest.main_class
        };

        args.push(main_class.to_owned());

        match manifest.arguments {
            Arguments::New { game, .. } => {
                for arg in game {
                    match arg {
                        JvmArgument::String(value) => {
                            args.push(value);
                        }
                        _ => break,
                    }
                }
            }
            Arguments::Old(arguments) => args.push(arguments),
        }

        if let Some(ref prof) = self.profile {
            prof.args.game.iter().for_each(|a| {
                dbg!(&a);
                args.push(a.to_owned());
            })
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

        Ok((args, classpath))
    }

    pub async fn launch(self) -> anyhow::Result<i32> {
        let game_dir = self.settings.game_dir.clone();
        let java = self.settings.java_bin.clone();
        let (args, classpath) = self.build_args()?;

        let mut process = dbg!(Command::new(java.get())
            .env("CLASSPATH", dbg!(classpath))
            .arg("-Xms2048M")
            .arg("-Xmx2048M")
            .args(dbg!(args))
            .current_dir(game_dir))
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
pub struct LaunchInstanceBuilder<S> {
    settings: S,
    profile: Option<LoaderProfile>,
}

impl LaunchInstanceBuilder<Undefined> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl LaunchInstanceBuilder<Undefined> {
    pub fn settings(self, settings: LaunchSettings) -> LaunchInstanceBuilder<LaunchSettings> {
        LaunchInstanceBuilder {
            settings,
            profile: self.profile,
        }
    }
}

impl<S> LaunchInstanceBuilder<S> {
    pub fn profile(self, profile: LoaderProfile) -> LaunchInstanceBuilder<S> {
        LaunchInstanceBuilder {
            settings: self.settings,
            profile: Some(profile),
        }
    }
}

impl LaunchInstanceBuilder<LaunchSettings> {
    pub fn build(self) -> LaunchInstance {
        LaunchInstance {
            settings: self.settings,
            profile: self.profile,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{instance::profile::read_json, repository::fabric_profile::FabricProfile};

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let mc_dir = std::env::current_dir().unwrap().join("minecraft");
        let settings = LaunchSettings {
            access_token: None,
            username: Username::new("ItWorks").unwrap(),
            uuid: None,
            assets: mc_dir.join("assets"),
            game_dir: mc_dir.clone(),
            java_bin: JavaRunner::default(),
            libraries_dir: mc_dir.clone().join("libraries"),
            manifest_file: mc_dir.clone().join("instances/1.18.2/1.18.2.json"),
            natives_dir: mc_dir.clone().join("instances/1.18.2/natives"),
            version_jar_file: mc_dir.join("instances/1.18.2/1.18.2.jar"),
            version: "1.18.2".to_string(),
            version_type: "release".to_string(),
        };

        let fabric = read_json::<FabricProfile>(
            "./minecraft/instances/1.18.2/fabric-loader-0.14.23-1.18.2.json",
        )
        .await
        .unwrap();

        let builder = LaunchInstanceBuilder::new()
            .settings(settings)
            .profile(fabric.into())
            .build();

        builder.launch().await.unwrap();
    }
}

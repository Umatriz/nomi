use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;
use tracing::{info, warn};

use crate::repository::{
    java_runner::JavaRunner,
    manifest::{read_manifest_from_file, Arguments, JvmArgument, ManifestLibrary},
    username::Username,
};
use rules::is_all_rules_satisfied;

use self::classpath::should_use_library;

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

    fn classpath(&self, libraries: Vec<ManifestLibrary>) -> anyhow::Result<(String, Vec<PathBuf>)> {
        let mut classpath = vec![];
        let mut native_libs = vec![];

        classpath.push(self.settings.version_jar_file.clone());

        for lib in libraries.iter() {
            if should_use_library(lib)? {
                if let Some(artifact) = lib.downloads.artifact.as_ref() {
                    let lib_path = artifact.path.clone().context("LibPath must be Some()")?;

                    let replaced_lib_path = if cfg!(target_os = "windows") {
                        lib_path.replace('/', "\\")
                    } else {
                        lib_path
                    };

                    let final_lib_path =
                        Path::new(&self.settings.libraries_dir).join(replaced_lib_path);

                    classpath.push(final_lib_path);
                }

                if let Some(natives) = lib.downloads.classifiers.as_ref() {
                    let native_option = match std::env::consts::OS {
                        "linux" => natives.natives_linux.as_ref(),
                        "windows" => natives.natives_windows.as_ref(),
                        "macos" => natives.natives_macos.as_ref(),
                        _ => unreachable!(),
                    };

                    if let Some(native_lib) = native_option {
                        let lib_path = native_lib.path.clone().context("LibPath must be Some()")?;

                        let replaced_lib_path = if cfg!(target_os = "windows") {
                            lib_path.replace('/', "\\")
                        } else {
                            lib_path
                        };

                        let final_lib_path = self.settings.libraries_dir.join(replaced_lib_path);

                        native_libs.push(final_lib_path.clone());
                        classpath.push(final_lib_path);
                    }
                }
            }
        }

        if let Some(extra_libs) = self.profile.as_ref().map(|p| &p.libraries) {
            extra_libs.iter().for_each(|lib| {
                classpath.push(self.settings.libraries_dir.join(&lib.jar));
            })
        }
        let classpath_iter = classpath.iter().map(|p| p.display().to_string());

        let final_classpath =
            itertools::intersperse(classpath_iter, CLASSPATH_SEPARATOR.to_string())
                .collect::<String>();

        Ok((final_classpath, native_libs))
    }

    fn process_natives(&self, natives: Vec<PathBuf>) -> anyhow::Result<()> {
        for lib in natives {
            let reader = OpenOptions::new().read(true).open(lib)?;
            std::fs::create_dir_all(&self.settings.natives_dir)?;

            let mut archive = zip::ZipArchive::new(reader)?;

            let mut names = vec![];
            archive
                .file_names()
                .map(String::from)
                .for_each(|el| names.push(el));

            names
                .into_iter()
                .filter(|l| l.ends_with(".dll") || l.ends_with(".so") || l.ends_with(".dylib"))
                .try_for_each(|lib| {
                    let mut file = archive.by_name(&lib)?;
                    let mut out = File::create(self.settings.natives_dir.join(lib))?;
                    io::copy(&mut file, &mut out)?;

                    Ok::<_, anyhow::Error>(())
                })?;
        }

        Ok(())
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

        let (classpath, native_libs) = self.classpath(manifest.libraries)?;

        self.process_natives(native_libs)?;

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
            args.push(format!(
                "-Djava.library.path={}",
                &self.settings.natives_dir.display()
            ));
            args.push("-Dminecraft.launcher.brand=${launcher_name}".into());
            args.push("-Dminecraft.launcher.version=${launcher_version}".into());
            args.push(format!(
                "-Dminecraft.client.jar={}",
                &self.settings.version_jar_file.display()
            ));
            args.push("-cp".to_string());
            args.push("${classpath}".to_string());
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
            Arguments::Old(arguments) => {
                let mut splited = arguments.split_whitespace().map(String::from).collect_vec();
                args.append(&mut splited);
            }
        };

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
                self.replace_args(
                    x,
                    &assets_dir,
                    &game_dir,
                    &natives_dir,
                    assets_index,
                    &classpath,
                )
            })
            .collect();

        Ok((args, classpath))
    }

    fn replace_args(
        &self,
        x: &String,
        assets_dir: &PathBuf,
        game_dir: &PathBuf,
        natives_dir: &PathBuf,
        assets_index: &String,
        classpath: &String,
    ) -> String {
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
            .replace("${classpath}", classpath)
    }

    pub async fn launch(self) -> anyhow::Result<i32> {
        let game_dir = self.settings.game_dir.clone();
        let java = self.settings.java_bin.clone();
        let native_dir = self.settings.natives_dir.clone();
        let (args, classpath) = self.build_args()?;

        let mut command = Command::new(java.get());
        command
            // .env("CLASSPATH", dbg!(classpath))
            .arg("-Xms2048M")
            .arg("-Xmx2048M")
            .args(dbg!(args))
            .current_dir(game_dir);
        println!("{:?}", command);
        let mut process = command.spawn().context("command failed to start")?;

        let status = process
            .wait()
            .await?
            .code()
            .context("can't get minecraft exit code")?;

        tokio::fs::remove_dir_all(native_dir).await?;

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

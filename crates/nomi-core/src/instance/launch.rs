use std::path::PathBuf;

use anyhow::Context;
use tokio::process::Command;

use crate::{
    launch::{rules::is_all_rules_satisfied, LaunchError, LaunchSettings},
    repository::manifest::{read_manifest_from_file, JvmArgument, ManifestLibrary},
};

use super::Undefined;

type InstanceClasspath =
    Box<dyn FnOnce(PathBuf, PathBuf, Vec<ManifestLibrary>) -> anyhow::Result<String>>;

pub struct LaunchInstance {
    settings: LaunchSettings,
    calsspath: Box<dyn FnOnce(PathBuf, PathBuf, Vec<ManifestLibrary>) -> anyhow::Result<String>>,
    main_class: Option<String>,
}

impl LaunchInstance {
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

        args.push(self.main_class.clone().unwrap_or(manifest.main_class));

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

        let classpath = (self.calsspath)(
            self.settings.version_jar_file.clone(),
            self.settings.libraries_dir.clone(),
            manifest.libraries,
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

#[derive(Default, Debug)]
pub struct LaunchInstanceBuilder<S, C, M> {
    settings: S,
    calsspath: C,
    main_class: M,
}

impl LaunchInstanceBuilder<Undefined, Undefined, Undefined> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C, M> LaunchInstanceBuilder<Undefined, C, M> {
    pub fn settings(self, settings: LaunchSettings) -> LaunchInstanceBuilder<LaunchSettings, C, M> {
        LaunchInstanceBuilder {
            settings,
            calsspath: self.calsspath,
            main_class: self.main_class,
        }
    }
}

impl<S, M> LaunchInstanceBuilder<S, Undefined, M> {
    pub fn classpath(
        self,
        classpath: impl FnOnce(PathBuf, PathBuf, Vec<ManifestLibrary>) -> anyhow::Result<String>
            + 'static,
    ) -> LaunchInstanceBuilder<S, InstanceClasspath, M> {
        LaunchInstanceBuilder {
            settings: self.settings,
            calsspath: Box::new(classpath),
            main_class: self.main_class,
        }
    }
}

impl<S, C> LaunchInstanceBuilder<S, C, Undefined> {
    pub fn main_class(
        self,
        main_class: Option<String>,
    ) -> LaunchInstanceBuilder<S, C, Option<String>> {
        LaunchInstanceBuilder {
            settings: self.settings,
            calsspath: self.calsspath,
            main_class,
        }
    }
}

impl LaunchInstanceBuilder<LaunchSettings, InstanceClasspath, Option<String>> {
    pub fn build(self) -> LaunchInstance {
        LaunchInstance {
            settings: self.settings,
            calsspath: self.calsspath,
            main_class: self.main_class,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        launch::{classpath::classpath, LaunchSettingsBuilder},
        loaders::maven::MavenData,
        repository::{fabric_profile::FabricProfile, library::SimpleLib},
    };

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
            .username("ItWorks".to_string())
            .uuid(None)
            .version("1.18.2".to_string())
            .version_jar_file(mc_dir.join("instances").join("1.18.2").join("1.18.2.jar"))
            .version_type("release".to_string())
            .build();
        // let settings = LaunchSettings {
        //     assets: mc_dir.join("assets"),
        //     access_token: None,
        //     username: "ItWorks".to_string(),
        //     uuid: None,
        //     game_dir: mc_dir.clone(),
        //     java_bin: "./java/jdk-17.0.8/bin/java.exe".into(),
        //     libraries_dir: mc_dir.clone().join("libraries"),
        //     manifest_file: mc_dir
        //         .clone()
        //         .join("instances")
        //         .join("1.18.2")
        //         .join("1.18.2.json"),
        //     natives_dir: mc_dir
        //         .clone()
        //         .join("instances")
        //         .join("1.18.2")
        //         .join("natives"),
        //     version: "1.18.2".to_string(),
        //     version_type: "release".to_string(),
        //     version_jar_file: mc_dir.join("instances").join("1.18.2").join("1.18.2.jar"),
        // };

        // assert_eq!(settings_one, settings);

        let fabric_libs: FabricProfile = async {
            let content = tokio::fs::read_to_string(
                "./minecraft/instances/1.18.2/fabric-loader-0.14.23-1.18.2.json",
            )
            .await
            .unwrap();

            serde_json::from_str(&content).unwrap()
        }
        .await;
        let libs = fabric_libs
            .libraries
            .iter()
            .map(|lib| MavenData::new(lib.name.as_str()))
            .map(SimpleLib::from)
            .collect_vec();

        let builder = LaunchInstanceBuilder::new()
            .settings(settings)
            .classpath(move |v, d, l| classpath(v, d, l, Some(libs)))
            .main_class(Some(fabric_libs.main_class))
            .build();

        builder.launch().await.unwrap();
    }
}

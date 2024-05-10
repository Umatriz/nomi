use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::info;

use crate::{
    instance::launch::macros::replace,
    repository::{
        java_runner::JavaRunner,
        manifest::{Argument, Arguments, Manifest, ManifestLibrary, Value},
        username::Username,
    },
    utils::path_to_string,
};
use rules::is_all_rules_satisfied;

use self::arguments::ArgumentsBuilder;

use super::{
    profile::{read_json, LoaderProfile},
    Undefined,
};

pub mod arguments;
pub mod rules;

#[cfg(windows)]
const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(windows))]
const CLASSPATH_SEPARATOR: &str = ":";

const LAUNCHER_NAME: &str = "nomi";
const LAUNCHER_VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
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

pub fn should_use_library(lib: &ManifestLibrary) -> bool {
    match lib.rules.as_ref() {
        Some(rules) => dbg!(is_all_rules_satisfied(rules)),
        None => true,
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LaunchInstance {
    pub settings: LaunchSettings,
    jvm_args: Option<Vec<String>>,
    loader_profile: Option<LoaderProfile>,
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

    fn process_natives(&self, natives: &[PathBuf]) -> anyhow::Result<()> {
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

    pub async fn launch(&self) -> anyhow::Result<()> {
        let manifest = read_json::<Manifest>(&self.settings.manifest_file).await?;

        let arguments_builder = ArgumentsBuilder::new(self, &manifest).finish();

        self.process_natives(arguments_builder.get_native_libs())?;

        let custom_jvm_arguments = arguments_builder.custom_jvm_arguments();
        let manifest_jvm_arguments = arguments_builder.manifest_jvm_arguments();
        let manifest_game_arguments = arguments_builder.manifest_game_arguments();

        let main_class = arguments_builder.get_main_class();

        let loader_arguments = arguments_builder.loader_arguments();

        let loader_jvm_arguments = loader_arguments.jvm_arguments();
        let loader_game_arguments = loader_arguments.game_arguments();

        let mut child = Command::new(self.settings.java_bin.get())
            .args(custom_jvm_arguments)
            .args(loader_jvm_arguments)
            .args(manifest_jvm_arguments)
            .arg(main_class)
            .args(manifest_game_arguments)
            .args(loader_game_arguments)
            .spawn()?;

        child
            .wait()
            .await?
            .code()
            .inspect(|code| info!("Minecraft exit code: {}", code));

        Ok(())
    }
}

pub mod macros {
    macro_rules! replace {
        (
            $initial:ident,
            $($name:literal => $value:expr),+
        ) => {
            $initial
            $(
               .replace($name, $value)
            )+
        };
    }
    pub(crate) use replace;
}

#[derive(Default)]
pub struct LaunchInstanceBuilder<S> {
    settings: S,
    jvm_args: Option<Vec<String>>,
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
            jvm_args: self.jvm_args,
            profile: self.profile,
        }
    }
}

impl<S> LaunchInstanceBuilder<S> {
    pub fn profile(self, profile: LoaderProfile) -> LaunchInstanceBuilder<S> {
        LaunchInstanceBuilder {
            settings: self.settings,
            jvm_args: self.jvm_args,
            profile: Some(profile),
        }
    }
}

impl<S> LaunchInstanceBuilder<S> {
    pub fn jvm_args(self, jvm_args: Vec<String>) -> LaunchInstanceBuilder<S> {
        LaunchInstanceBuilder {
            settings: self.settings,
            jvm_args: Some(jvm_args),
            profile: self.profile,
        }
    }
}

impl LaunchInstanceBuilder<LaunchSettings> {
    pub fn build(self) -> LaunchInstance {
        LaunchInstance {
            settings: self.settings,
            jvm_args: self.jvm_args,
            loader_profile: self.profile,
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

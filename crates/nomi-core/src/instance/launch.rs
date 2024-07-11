use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use arguments::UserData;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{debug, info, trace, warn};

use crate::{
    downloads::Assets,
    fs::read_json_config,
    repository::{
        java_runner::JavaRunner,
        manifest::{Manifest, VersionType},
    },
};

use self::arguments::ArgumentsBuilder;

use super::{profile::LoaderProfile, Undefined};

pub mod arguments;
pub mod rules;

#[cfg(windows)]
const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(windows))]
const CLASSPATH_SEPARATOR: &str = ":";

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Hash)]
pub struct LaunchSettings {
    pub assets: PathBuf,
    pub java_bin: JavaRunner,
    pub game_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub natives_dir: PathBuf,
    pub version_jar_file: PathBuf,

    pub version: String,
    pub version_type: VersionType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct LaunchInstance {
    pub settings: LaunchSettings,
    jvm_args: Option<Vec<String>>,
    loader_profile: Option<LoaderProfile>,
}

impl LaunchInstance {
    #[tracing::instrument(err)]
    pub async fn delete(&self, delete_client: bool, delete_libraries: bool, delete_assets: bool) -> anyhow::Result<()> {
        let manifest = read_json_config::<Manifest>(&self.settings.manifest_file).await?;
        let arguments_builder = ArgumentsBuilder::new(self, &manifest).with_classpath();

        if delete_client {
            let _ = tokio::fs::remove_file(&self.settings.version_jar_file)
                .await
                .inspect(|()| {
                    debug!("Removed client successfully: {}", &self.settings.version_jar_file.display());
                })
                .inspect_err(|_| {
                    warn!("Cannot remove client: {}", &self.settings.version_jar_file.display());
                });
        }

        if delete_libraries {
            for library in arguments_builder.classpath_as_slice() {
                let _ = tokio::fs::remove_file(library)
                    .await
                    .inspect(|()| trace!("Removed library successfully: {}", library.display()))
                    .inspect_err(|_| warn!("Cannot remove library: {}", library.display()));
            }
        }

        if delete_assets {
            let assets = read_json_config::<Assets>(dbg!(&self
                .settings
                .assets
                .join("indexes")
                .join(format!("{}.json", manifest.asset_index.id))))
            .await?;
            for asset in assets.objects.values() {
                let path = &self.settings.assets.join("objects").join(&asset.hash[0..2]).join(&asset.hash);

                let _ = tokio::fs::remove_file(path)
                    .await
                    .inspect(|()| trace!("Removed asset successfully: {}", path.display()))
                    .inspect_err(|e| warn!("Cannot remove asset: {}. Error: {e}", path.display()));
            }
        }

        Ok(())
    }

    pub fn loader_profile(&self) -> Option<&LoaderProfile> {
        self.loader_profile.as_ref()
    }

    fn process_natives(&self, natives: &[PathBuf]) -> anyhow::Result<()> {
        for lib in natives {
            let reader = OpenOptions::new().read(true).open(lib)?;
            std::fs::create_dir_all(&self.settings.natives_dir)?;

            let mut archive = zip::ZipArchive::new(reader)?;

            let mut names = vec![];
            archive.file_names().map(String::from).for_each(|el| names.push(el));

            names
                .into_iter()
                .filter(|l| {
                    let path = Path::new(l).extension();

                    let check = |expected_ext| path.map_or(false, |ext| ext.eq_ignore_ascii_case(expected_ext));

                    check("dll") || check("so") || check("dylib")
                })
                .try_for_each(|lib| {
                    let mut file = archive.by_name(&lib)?;
                    let mut out = File::create(self.settings.natives_dir.join(lib))?;
                    io::copy(&mut file, &mut out)?;

                    Ok::<_, anyhow::Error>(())
                })?;
        }

        Ok(())
    }

    pub async fn launch(&self, user_data: UserData, java_runner: &JavaRunner) -> anyhow::Result<()> {
        let manifest = read_json_config::<Manifest>(&self.settings.manifest_file).await?;

        let arguments_builder = ArgumentsBuilder::new(self, &manifest).with_classpath().with_userdata(user_data);

        self.process_natives(arguments_builder.get_native_libs())?;

        let custom_jvm_arguments = arguments_builder.custom_jvm_arguments();
        let manifest_jvm_arguments = arguments_builder.manifest_jvm_arguments();
        let manifest_game_arguments = arguments_builder.manifest_game_arguments();

        let main_class = arguments_builder.get_main_class();

        let loader_arguments = arguments_builder.loader_arguments();

        let loader_jvm_arguments = loader_arguments.jvm_arguments();
        let loader_game_arguments = loader_arguments.game_arguments();

        let mut child = Command::new(java_runner.get())
            .args(custom_jvm_arguments)
            .args(loader_jvm_arguments)
            .args(dbg!(manifest_jvm_arguments))
            .arg(main_class)
            .args(dbg!(manifest_game_arguments))
            .args(loader_game_arguments)
            .spawn()?;

        child.wait().await?.code().inspect(|code| info!("Minecraft exit code: {}", code));

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
#[must_use]
#[allow(clippy::module_name_repetitions)]
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
    #[must_use]
    pub fn build(self) -> LaunchInstance {
        LaunchInstance {
            settings: self.settings,
            jvm_args: self.jvm_args,
            loader_profile: self.profile,
        }
    }
}

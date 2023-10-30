use anyhow::Context;
use reqwest::Client;
use thiserror::Error;
use tokio::sync::{Mutex, OnceCell};

use crate::{
    configs::{
        profile::VersionProfilesConfig, read_toml_config, user::Settings, variables::Variables,
        write_toml_config,
    },
    repository::{
        launcher_manifest::{LauncherManifest, LauncherManifestVersion},
        manifest::Manifest,
    },
};

use super::get_launcher_manifest;

pub static VARIABLES_STATE: OnceCell<Variables> = OnceCell::const_new();

pub async fn variables_state_try_init() -> anyhow::Result<Variables> {
    let current = std::env::current_dir()?;
    let path = current.join("./.nomi/Variables.toml");
    match path.exists() {
        true => Ok(read_toml_config(path).await?),
        false => {
            let data = Variables { root: current };
            write_toml_config(&data, path).await?;
            Ok(data)
        }
    }
}

pub static PROFILES_STATE: Mutex<OnceCell<VersionProfilesConfig>> =
    Mutex::const_new(OnceCell::const_new());

pub async fn profiles_state_try_init() -> anyhow::Result<VersionProfilesConfig> {
    let current = std::env::current_dir()?;
    let path = current.join("./.nomi/Profiles.toml");
    match path.exists() {
        true => Ok(read_toml_config(path).await?),
        false => Ok(VersionProfilesConfig { profiles: vec![] }),
    }
}

pub static SETTINGS_STATE: OnceCell<Settings> = OnceCell::const_new();

pub async fn settings_state_try_init() -> anyhow::Result<Settings> {
    let current = std::env::current_dir()?;
    let path = current.join("./.nomi/Settings.toml");
    match path.exists() {
        true => read_toml_config(path).await,
        false => Err(SettingsStateError::NotFound.into()),
    }
}

#[derive(Error, Debug)]
pub enum SettingsStateError {
    #[error("`.nomi/Settings.toml` does not exists")]
    NotFound,
}

pub static LAUNCHER_MANIFEST_STATE: OnceCell<ManifestState> = OnceCell::const_new();

pub async fn launcher_manifest_state_try_init() -> anyhow::Result<ManifestState> {
    Ok(ManifestState {
        launcher: get_launcher_manifest().await?,
    })
}

#[derive(Debug, Default)]
pub struct ManifestState {
    pub launcher: LauncherManifest,
}

impl ManifestState {
    pub fn find_version(&self, version: impl Into<String>) -> Option<&LauncherManifestVersion> {
        let version = version.into();
        self.launcher.versions.iter().find(|v| v.id == version)
    }

    pub async fn get_version_manifest(
        &self,
        version: impl Into<String>,
    ) -> anyhow::Result<Manifest> {
        let url = &self
            .find_version(version)
            .context("cannot find such version")?
            .url;

        super::get(url).await
    }

    pub async fn get_version_manifest_content(
        &self,
        version: impl Into<String>,
    ) -> anyhow::Result<String> {
        let url = &self
            .find_version(version)
            .context("cannot find such version")?
            .url;

        Ok(Client::new().get(url).send().await?.text().await?)
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::*;

    #[tokio::test]
    async fn manifest_init_test() {
        let sub = tracing_subscriber::fmt()
            .compact()
            .with_max_level(Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(sub).unwrap();

        let m = LAUNCHER_MANIFEST_STATE
            .get_or_try_init(launcher_manifest_state_try_init)
            .await
            .unwrap();
        println!("{:?}", &m.launcher.versions[..5]);
        println!(
            "{:?}",
            &LAUNCHER_MANIFEST_STATE
                .get_or_try_init(launcher_manifest_state_try_init)
                .await
                .unwrap()
                .launcher
                .versions[..5]
        );
    }

    #[tokio::test]
    async fn variables_init_test() {
        let m = VARIABLES_STATE
            .get_or_try_init(variables_state_try_init)
            .await
            .unwrap();
        assert_eq!(m.root, std::env::current_dir().unwrap());
        dbg!(m);
    }

    #[tokio::test]
    async fn profiles_init_test() {
        let binding = PROFILES_STATE.try_lock().unwrap();
        let p = binding
            .get_or_try_init(profiles_state_try_init)
            .await
            .unwrap();
        dbg!(p);
    }
}

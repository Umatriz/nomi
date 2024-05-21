use anyhow::Context;
use reqwest::{get, Client};
use tokio::sync::OnceCell;

use crate::repository::{
    launcher_manifest::{LauncherManifest, LauncherManifestVersion},
    manifest::Manifest,
};

// TODO: Write helper functions for quick access

pub const LAUNCHER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
pub static LAUNCHER_MANIFEST_STATE: OnceCell<ManifestState> = OnceCell::const_new();

pub async fn get_launcher_manifest() -> anyhow::Result<LauncherManifest> {
    tracing::debug!("Calling Launcher Manifest");
    Ok(Client::new()
        .get(LAUNCHER_MANIFEST)
        .send()
        .await?
        .json::<LauncherManifest>()
        .await?)
}

pub async fn get_launcher_manifest_state() -> anyhow::Result<&'static ManifestState> {
    LAUNCHER_MANIFEST_STATE
        .get_or_try_init(launcher_manifest_state_try_init)
        .await
}

async fn launcher_manifest_state_try_init() -> anyhow::Result<ManifestState> {
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

        get(url).await?.json().await.map_err(|err| err.into())
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

// #[cfg(test)]
// mod tests {
//     use tracing::Level;

//     use super::*;

//     #[tokio::test]
//     async fn manifest_init_test() {
//         let sub = tracing_subscriber::fmt()
//             .compact()
//             .with_max_level(Level::DEBUG)
//             .finish();
//         tracing::subscriber::set_global_default(sub).unwrap();

//         let m = LAUNCHER_MANIFEST_STATE
//             .get_or_try_init(launcher_manifest_state_try_init)
//             .await
//             .unwrap();
//         println!("{:?}", &m.launcher.versions[..5]);
//         println!(
//             "{:?}",
//             &LAUNCHER_MANIFEST_STATE
//                 .get_or_try_init(launcher_manifest_state_try_init)
//                 .await
//                 .unwrap()
//                 .launcher
//                 .versions[..5]
//         );
//     }

//     #[tokio::test]
//     async fn variables_init_test() {
//         let m = VARIABLES_STATE
//             .get_or_try_init(variables_state_try_init)
//             .await
//             .unwrap();
//         assert_eq!(m.root, std::env::current_dir().unwrap());
//         dbg!(m);
//     }

//     #[tokio::test]
//     async fn profiles_init_test() {
//         let binding = PROFILES_STATE.try_lock().unwrap();
//         let p = binding
//             .get_or_try_init(profiles_state_try_init)
//             .await
//             .unwrap();
//         dbg!(p);
//     }
// }

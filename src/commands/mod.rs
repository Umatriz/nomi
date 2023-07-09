use crate::{
    bootstrap::{ClientAuth, ClientBootstrap, ClientSettings, ClientVersion},
    configs::launcher::Launcher,
    downloads::{
        launcher_manifest::{LauncherManifest, LauncherManifestVersion},
        Download,
    },
    utils::GetPath,
};

use serde::Serialize;
use thiserror::Error;
use anyhow::Result;


#[derive(Serialize, Clone)]
struct Downloading {
    state: bool,
}

pub async fn download_version(_id: String) -> Result<()> {
    let _load: Download = Download::new().await?;

    // load.download(id, GetPath::game().to_str().unwrap().to_string())
    //   .await
    //   .unwrap();

    println!("1");
    tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
    println!("2");

    Ok(())
}

pub async fn get_manifest() -> Result<Vec<LauncherManifestVersion>> {
    let resp: LauncherManifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .map_err(CommandsError::FailedToDownloadManifest)?
            .json()
            .await
            .map_err(CommandsError::CantParseManifestToJson)?;

    Ok(resp.versions)
}

pub async fn get_config() -> Result<Launcher> {
    let launcher_config = Launcher::from_file(None)?;

    return Ok(launcher_config);
}

pub async fn launch(username: String, version: String) -> Result<()> {
    let bootstrap = ClientBootstrap::new(ClientSettings {
        assets: GetPath::game()?.join("assets"),
        auth: ClientAuth {
            username,
            access_token: None,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        },
        game_dir: GetPath::game()?,
        java_bin: GetPath::get_java_bin()?,
        libraries_dir: GetPath::game()?.join("libraries"),
        manifest_file: GetPath::game()?
            .join("versions")
            .join(&version)
            .join(format!("{}.json", version)),
        natives_dir: GetPath::game()?
            .join("versions")
            .join(&version)
            .join("natives"),
        version: ClientVersion {
            version: version.clone(),
            version_type: "release".to_string(),
        },
        version_jar_file: GetPath::game()?
            .join("versions")
            .join(&version)
            .join(format!("{}.jar", version)),
    });

    bootstrap.launch()?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum CommandsError {
  #[error("Failed to download minecraft manifest file")]
  FailedToDownloadManifest(reqwest::Error),

  #[error("Can't parse minecraft manifest file to json")]
  CantParseManifestToJson(reqwest::Error),
}
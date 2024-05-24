use std::sync::Arc;

use anyhow::anyhow;
use nomi_core::{
    configs::profile::{
        Loader, ProfileState, VersionProfile, VersionProfileBuilder, VersionProfilesConfig,
    },
    downloads::{
        traits::{DownloadResult, Downloader, DownloaderIO, DownloaderIOExt},
        DownloadQueue,
    },
    fs::{read_toml_config, write_toml_config},
    game_paths::GamePaths,
    instance::{launch::LaunchSettings, InstanceBuilder},
    loaders::{fabric::Fabric, vanilla::Vanilla},
    repository::{java_runner::JavaRunner, manifest::VersionType, username::Username},
};
use tokio::sync::mpsc::Sender;

use crate::utils::Crash;

pub fn spawn_download(
    profile: Arc<VersionProfile>,
    result_tx: Sender<VersionProfile>,
    progress_tx: tokio::sync::mpsc::Sender<DownloadResult>,
    total_tx: tokio::sync::mpsc::Sender<u32>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let data = try_download(profile, progress_tx.clone(), total_tx)
            .await
            .crash();

        let _ = result_tx.send(data).await;
    })
}

async fn try_download(
    profile: Arc<VersionProfile>,
    sender: tokio::sync::mpsc::Sender<DownloadResult>,
    total_tx: tokio::sync::mpsc::Sender<u32>,
) -> anyhow::Result<VersionProfile> {
    let current_dir = std::env::current_dir()?;
    let mc_dir: std::path::PathBuf = current_dir.join("minecraft");

    let ProfileState::NotDownloaded {
        version,
        version_type,
        loader,
    } = &profile.state
    else {
        return Err(anyhow!("This profile is already downloaded"));
    };

    let game_paths = GamePaths {
        game: mc_dir.clone(),
        assets: mc_dir.join("assets"),
        version: mc_dir.join("versions").join(&profile.version()),
        libraries: mc_dir.join("libraries"),
    };

    let builder = InstanceBuilder::new()
        .name(profile.name.clone())
        .version(profile.version().to_string())
        .game_paths(game_paths.clone())
        .sender(sender.clone());

    let instance = match loader {
        Loader::Vanilla => builder.instance(Box::new(
            Vanilla::new(profile.version(), game_paths.clone()).await?,
        )),
        Loader::Fabric => builder.instance(Box::new(
            Fabric::new(profile.version(), None::<String>, game_paths.clone()).await?,
        )),
    }
    .build();

    let settings = LaunchSettings {
        access_token: None,
        username: Username::default(),
        uuid: None,
        assets: instance.game_paths.assets.clone(),
        game_dir: instance.game_paths.game.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: instance.game_paths.libraries.clone(),
        manifest_file: instance
            .game_paths
            .version
            .join(format!("{}.json", &version)),
        natives_dir: instance.game_paths.version.join("natives"),
        version_jar_file: instance
            .game_paths
            .version
            .join(format!("{}.jar", &version)),
        version: version.to_string(),
        version_type: version_type.clone(),
    };

    let launch_instance = instance.launch_instance(
        settings,
        Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]),
    );

    let assets = instance.assets().await?;

    assets.get_io().io().await?;

    let downloader: Box<dyn Downloader<Data = DownloadResult>> =
        instance.instance().into_downloader();

    let downloader = DownloadQueue::new()
        .with_downloader(assets)
        .with_downloader_dyn(downloader);

    let _ = total_tx.send(downloader.total()).await;

    Box::new(downloader).download(sender).await;

    let profile = VersionProfile {
        id: profile.id,
        name: profile.name.clone(),
        state: ProfileState::Downloaded(Box::new(launch_instance)),
    };

    Ok(profile)
}

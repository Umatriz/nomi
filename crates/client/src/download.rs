use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use egui_task_manager::TaskProgressShared;
use nomi_core::{
    configs::profile::{Loader, ProfileState},
    downloads::{
        progress::MappedSender,
        traits::{DownloadResult, Downloader},
        AssetsDownloader, DownloadQueue,
    },
    game_paths::GamePaths,
    instance::{launch::LaunchSettings, Instance},
    loaders::{fabric::Fabric, vanilla::Vanilla},
    repository::java_runner::JavaRunner,
    state::get_launcher_manifest,
};
use parking_lot::RwLock;

use crate::{errors_pool::ErrorPoolExt, views::ModdedProfile};

pub async fn task_download_version(profile: Arc<RwLock<ModdedProfile>>, progress_shared: TaskProgressShared) -> Option<()> {
    try_download_version(profile, progress_shared).await.report_error()
}

async fn try_download_version(profile: Arc<RwLock<ModdedProfile>>, progress_shared: TaskProgressShared) -> anyhow::Result<()> {
    let launch_instance = {
        let mc_dir = PathBuf::from("./minecraft");

        let version_profile = {
            let version_profile = &profile.read().profile;
            version_profile.clone()
        };

        let ProfileState::NotDownloaded {
            version,
            version_type,
            loader,
        } = &version_profile.state
        else {
            return Err(anyhow!("This profile is already downloaded"));
        };

        let game_paths = GamePaths {
            game: mc_dir.clone(),
            assets: mc_dir.join("assets"),
            version: mc_dir.join("versions").join(version),
            libraries: mc_dir.join("libraries"),
        };

        let builder = Instance::builder()
            .name(version_profile.name.clone())
            .version(version_profile.version().to_string())
            .game_paths(game_paths.clone());

        let instance = match loader {
            Loader::Vanilla => builder.instance(Box::new(Vanilla::new(version_profile.version(), game_paths.clone()).await?)),
            Loader::Fabric { version } => builder.instance(Box::new(
                Fabric::new(version_profile.version(), version.as_ref(), game_paths.clone()).await?,
            )),
        }
        .build();

        let settings = LaunchSettings {
            assets: instance.game_paths.assets.clone(),
            game_dir: instance.game_paths.game.clone(),
            java_bin: JavaRunner::default(),
            libraries_dir: instance.game_paths.libraries.clone(),
            manifest_file: instance.game_paths.version.join(format!("{}.json", &version)),
            natives_dir: instance.game_paths.version.join("natives"),
            version_jar_file: instance.game_paths.version.join(format!("{}.jar", &version)),
            version: version.to_string(),
            version_type: version_type.clone(),
        };

        let launch_instance = instance.launch_instance(settings, Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]));

        let instance = instance.instance();

        let io = instance.io();

        let downloader: Box<dyn Downloader<Data = DownloadResult>> = instance.into_downloader();

        io.await?;

        let downloader = DownloadQueue::new().with_downloader_dyn(downloader);

        let _ = progress_shared.set_total(downloader.total());

        let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress_shared.sender()));

        Box::new(downloader).download(&mapped_sender).await;

        launch_instance
    };

    profile.write().profile.state = ProfileState::downloaded(launch_instance);

    Ok(())
}

pub async fn task_assets(version: String, assets_dir: PathBuf, progress_shared: TaskProgressShared) -> Option<()> {
    try_assets(version, assets_dir, progress_shared).await.report_error()
}

async fn try_assets(version: String, assets_dir: PathBuf, progress_shared: TaskProgressShared) -> anyhow::Result<()> {
    let manifest = get_launcher_manifest().await?;
    let version_manifest = manifest.get_version_manifest(version).await?;

    let downloader = AssetsDownloader::new(
        version_manifest.asset_index.url,
        version_manifest.asset_index.id,
        assets_dir.join("objects"),
        assets_dir.join("indexes"),
    )
    .await?;

    let io = downloader.io();

    let _ = progress_shared.set_total(downloader.total());

    let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress_shared.sender()));

    Box::new(downloader).download(&mapped_sender).await;

    io.await?;

    Ok(())
}

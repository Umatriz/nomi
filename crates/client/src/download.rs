use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use eframe::egui::Context;
use egui_task_manager::TaskProgressShared;
use nomi_core::{
    configs::profile::{Loader, ProfileState},
    downloads::{progress::MappedSender, traits::Downloader, AssetsDownloader, DownloadQueue},
    game_paths::GamePaths,
    instance::{launch::LaunchSettings, Profile},
    loaders::{
        combined::VanillaCombinedDownloader,
        fabric::Fabric,
        forge::{Forge, ForgeVersion},
    },
    repository::java_runner::JavaRunner,
    state::get_launcher_manifest,
};
use parking_lot::RwLock;

use crate::{errors_pool::ErrorPoolExt, views::ModdedProfile};

pub async fn task_download_version(
    progress_shared: TaskProgressShared,
    ctx: Context,
    profile: Arc<RwLock<ModdedProfile>>,
    java_runner: JavaRunner,
) -> Option<()> {
    try_download_version(progress_shared, ctx, profile, java_runner).await.report_error()
}

async fn try_download_version(
    progress_shared: TaskProgressShared,
    ctx: Context,
    profile: Arc<RwLock<ModdedProfile>>,
    java_runner: JavaRunner,
) -> anyhow::Result<()> {
    let launch_instance = {
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

        let game_paths = GamePaths::from_id(version_profile.id);

        let builder = Profile::builder()
            .name(version_profile.name.clone())
            .version(version_profile.version().to_string())
            .game_paths(game_paths.clone());

        let combined_downloader = VanillaCombinedDownloader::new(version_profile.version(), game_paths.clone()).await?;
        let instance = match loader {
            Loader::Vanilla => builder.downloader(Box::new(combined_downloader)),
            Loader::Fabric { version } => {
                let combined = combined_downloader
                    .with_loader(|game_version, game_paths| Fabric::new(game_version, version.as_ref(), game_paths))
                    .await?;
                builder.downloader(Box::new(combined))
            }
            Loader::Forge => {
                let combined = combined_downloader
                    .with_loader(|game_version, game_paths| Forge::new(game_version, ForgeVersion::Recommended, game_paths, java_runner))
                    .await?;
                builder.downloader(Box::new(combined))
            }
        }
        .build();

        let settings = LaunchSettings {
            java_runner: None,
            version: version.to_string(),
            version_type: version_type.clone(),
        };

        let launch_instance = instance.launch_instance(settings, Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]));

        let instance = instance.downloader();
        let io = instance.io();
        let downloader = instance.into_downloader();

        let _ = progress_shared.set_total(downloader.total());

        let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress_shared.sender())).with_side_effect(move || ctx.request_repaint());
        downloader.download(&mapped_sender).await;

        io.await?;

        launch_instance
    };

    profile.write().profile.state = ProfileState::downloaded(launch_instance);

    Ok(())
}

pub async fn task_assets(progress_shared: TaskProgressShared, ctx: Context, version: String, assets_dir: PathBuf) -> Option<()> {
    try_assets(progress_shared, ctx, version, assets_dir).await.report_error()
}

async fn try_assets(progress_shared: TaskProgressShared, ctx: Context, version: String, assets_dir: PathBuf) -> anyhow::Result<()> {
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

    let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress_shared.sender())).with_side_effect(move || ctx.request_repaint());

    Box::new(downloader).download(&mapped_sender).await;

    io.await?;

    Ok(())
}

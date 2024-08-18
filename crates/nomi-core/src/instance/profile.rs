use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use crate::{downloads::downloaders::assets::AssetsDownloader, game_paths::GamePaths, state::get_launcher_manifest};

use super::{
    launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettings},
    marker::ProfileDownloader,
};

#[derive(Debug, TypedBuilder)]
pub struct Profile {
    downloader: Box<dyn ProfileDownloader>,
    pub game_paths: GamePaths,
    pub version: String,
    pub name: String,
}

impl Profile {
    pub fn downloader(self) -> Box<dyn ProfileDownloader> {
        self.downloader
    }

    pub async fn assets(&self) -> anyhow::Result<AssetsDownloader> {
        let manifest = get_launcher_manifest().await?;
        let version_manifest = manifest.get_version_manifest(&self.version).await?;

        AssetsDownloader::new(
            version_manifest.asset_index.url,
            version_manifest.asset_index.id,
            self.game_paths.assets.join("objects"),
            self.game_paths.assets.join("indexes"),
        )
        .await
    }

    #[must_use]
    pub fn launch_instance(&self, settings: LaunchSettings, jvm_args: Option<Vec<String>>) -> LaunchInstance {
        let builder = LaunchInstanceBuilder::new().settings(settings);
        let builder = match jvm_args {
            Some(jvm) => builder.jvm_args(jvm),
            None => builder,
        };

        self.downloader.insert(builder).build()
    }
}

#[tracing::instrument]
pub async fn delete_profile(paths: GamePaths, game_version: &str) {
    let path = paths.version_jar_file(game_version);
    let _ = tokio::fs::remove_file(&path)
        .await
        .inspect(|()| {
            debug!("Removed client successfully: {}", &path.display());
        })
        .inspect_err(|_| {
            warn!("Cannot remove client: {}", &path.display());
        });

    let path = &paths.profile_config();
    let _ = tokio::fs::remove_file(&path)
        .await
        .inspect(|()| {
            debug!(path = %&path.display(), "Removed profile config successfully");
        })
        .inspect_err(|_| {
            warn!(path = %&path.display(), "Cannot profile config");
        });

    let path = &paths.profile;
    let _ = tokio::fs::remove_dir(&path)
        .await
        .inspect(|()| {
            debug!(path = %&path.display(), "Removed profile directory successfully");
        })
        .inspect_err(|_| {
            warn!(path = %&path.display(), "Cannot profile directory");
        });
}

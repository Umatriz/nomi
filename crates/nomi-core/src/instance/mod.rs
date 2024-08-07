use typed_builder::TypedBuilder;

pub mod builder_ext;
pub mod launch;
pub mod logs;
pub mod profile;
pub mod version_marker;

use crate::{downloads::downloaders::assets::AssetsDownloader, game_paths::GamePaths, state::get_launcher_manifest};

use self::{
    launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettings},
    version_marker::Version,
};

#[derive(Debug, TypedBuilder)]
pub struct Instance {
    instance: Box<dyn Version>,
    pub game_paths: GamePaths,
    pub version: String,
    pub name: String,
}

impl Instance {
    pub fn instance(self) -> Box<dyn Version> {
        self.instance
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

        self.instance.insert(builder).build()
    }
}

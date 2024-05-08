use const_typed_builder::Builder;
use std::path::PathBuf;
use tokio::sync::mpsc::Receiver;
use tracing::info;

pub mod builder_ext;
pub mod launch;
pub mod profile;
pub mod version_marker;

use crate::{
    downloads::assets::AssetsDownloader,
    utils::state::{launcher_manifest_state_try_init, LAUNCHER_MANIFEST_STATE},
};

use self::{
    launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettings},
    version_marker::Version,
};

#[derive(Default, Debug)]
pub struct Undefined;

#[derive(Debug, Builder)]
pub struct Instance {
    instance: Box<dyn Version>,
    pub version: String,
    pub libraries: PathBuf,
    pub version_path: PathBuf,
    pub assets: PathBuf,
    pub game: PathBuf,
    pub name: String,
}

impl Instance {
    pub async fn download(&self) -> anyhow::Result<()> {
        self.instance
            .download(&self.version_path, &self.version)
            .await?;
        self.instance.download_libraries(&self.libraries).await?;
        self.instance.create_json(&self.version_path).await?;

        Ok(())
    }

    pub async fn assets(&self) -> anyhow::Result<AssetsInstance> {
        let manifest = LAUNCHER_MANIFEST_STATE
            .get_or_try_init(launcher_manifest_state_try_init)
            .await?;
        let version_manifest = manifest.get_version_manifest(&self.version).await?;

        Ok(AssetsInstanceBuilder::new()
            .downloader(
                AssetsDownloader::new(
                    version_manifest.asset_index.url,
                    version_manifest.asset_index.id,
                )
                .await?,
            )
            .indexes(self.assets.join("indexes"))
            .objects(self.assets.join("objects"))
            .build())
    }

    pub fn launch_instance(
        self,
        settings: LaunchSettings,
        jvm_args: Option<Vec<String>>,
    ) -> LaunchInstance {
        let builder = LaunchInstanceBuilder::new().settings(settings);
        let builder = match jvm_args {
            Some(jvm) => builder.jvm_args(jvm),
            None => builder,
        };
        self.instance.insert(builder).build()
    }
}

#[derive(Builder)]
pub struct AssetsInstance {
    downloader: AssetsDownloader,
    objects: PathBuf,
    indexes: PathBuf,
}

impl AssetsInstance {
    pub async fn download(self) -> anyhow::Result<()> {
        self.downloader
            .download_assets_chunked(&self.objects)
            .await?;
        info!("Assets downloaded successfully");

        self.downloader.get_assets_json(&self.indexes).await?;
        info!("Assets json created successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::loaders::{fabric::Fabric, vanilla::Vanilla};

    use super::*;

    #[tokio::test]
    async fn build_test() {
        let _builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Box::new(Vanilla::new("1.18.2").await.unwrap()))
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();
    }

    #[tokio::test]
    async fn assets_test() {
        let builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Box::new(Vanilla::new("1.18.2").await.unwrap()))
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();

        builder.assets().await.unwrap().download().await.unwrap();
    }

    #[tokio::test]
    async fn fabric_test() {
        let subscriber = tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::INFO)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Box::new(
                Fabric::new("1.18.2", None::<String>).await.unwrap(),
            ))
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();

        // builder.assets().await.unwrap().download().await.unwrap();
        // builder.assets().and_then(|i| i.download()).await.unwrap();

        builder.download().await.unwrap();
    }
}

use const_typed_builder::Builder;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tracing::info;

pub mod builder_ext;
pub mod launch;
pub mod profile;
pub mod version_marker;

use crate::{
    downloads::{
        downloadable::{DownloadResult, DownloaderIO},
        downloaders::assets::AssetsDownloader,
    },
    game_paths::GamePaths,
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
    sender: Sender<DownloadResult>,
    pub game_paths: GamePaths,
    pub version: String,
    pub name: String,
}

impl Instance {
    pub async fn download(self) -> anyhow::Result<()> {
        // self.instance
        //     .download(&self.game_paths.version, &self.version)
        //     .await?;
        // self.instance
        //     .download_libraries(&self.game_paths.libraries)
        //     .await?;
        // self.instance.create_json(&self.game_paths.version).await?;

        {
            let io = self.instance.get_io_dyn();
            io.io().await?;
        }

        self.instance.download(self.sender.clone()).await;

        Ok(())
    }

    pub async fn assets(&self) -> anyhow::Result<AssetsDownloader> {
        let manifest = LAUNCHER_MANIFEST_STATE
            .get_or_try_init(launcher_manifest_state_try_init)
            .await?;
        let version_manifest = manifest.get_version_manifest(&self.version).await?;

        AssetsDownloader::new(
            version_manifest.asset_index.url,
            version_manifest.asset_index.id,
            self.game_paths.assets.join("objects"),
            self.game_paths.assets.join("indexes"),
        )
        .await
    }

    pub fn launch_instance(
        &self,
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

// #[cfg(test)]
// mod tests {
//     use tracing::debug;

//     use crate::{
//         downloads::downloadable::Downloader,
//         game_paths::GamePaths,
//         loaders::{fabric::Fabric, vanilla::Vanilla},
//     };

//     use super::*;

//     #[tokio::test]
//     async fn assets_test() {
//         let game_paths = GamePaths {
//             game: "./minecraft".into(),
//             assets: "./minecraft/assets".into(),
//             version: "./minecraft/instances/1.18.2".into(),
//             libraries: "./minecraft/libraries".into(),
//         };

//         let (tx, _) = tokio::sync::mpsc::channel(100);
//         let builder = InstanceBuilder::new()
//             .version("1.18.2".into())
//             .game_paths(game_paths)
//             .instance(Box::new(Vanilla::new("1.18.2").await.unwrap()))
//             .name("1.18.2-minecraft".into())
//             .sender(tx.clone())
//             .build();

//         Box::new(builder.assets().await.unwrap()).download(tx).await;
//     }

//     #[tokio::test]
//     async fn fabric_test() {
//         let subscriber = tracing_subscriber::fmt()
//             .pretty()
//             .with_max_level(tracing::Level::INFO)
//             .finish();
//         tracing::subscriber::set_global_default(subscriber).unwrap();

//         let (tx, mut rx) = tokio::sync::mpsc::channel(100);

//         tokio::spawn(async move {
//             while let Some(result) = rx.recv().await {
//                 debug!("{:?}", result);
//             }
//         });

//         let game_paths = GamePaths {
//             game: "./minecraft".into(),
//             assets: "./minecraft/assets".into(),
//             version: "./minecraft/instances/1.18.2".into(),
//             libraries: "./minecraft/libraries".into(),
//         };

//         let builder = InstanceBuilder::new()
//             .version("1.18.2".into())
//             .game_paths(game_paths.clone())
//             .instance(Box::new(
//                 Fabric::new("1.18.2", None::<String>, game_paths)
//                     .await
//                     .unwrap(),
//             ))
//             .name("1.18.2-minecraft".into())
//             .sender(tx)
//             .build();

//         // builder.assets().await.unwrap().download().await.unwrap();
//         // builder.assets().and_then(|i| i.download()).await.unwrap();

//         builder.download().await.unwrap();
//     }
// }

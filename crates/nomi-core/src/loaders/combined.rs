use std::{fmt::Debug, future::Future};

use crate::{
    downloads::{
        progress::ProgressSender,
        traits::{DownloadResult, Downloader},
        DownloadQueue,
    },
    game_paths::GamePaths,
    instance::marker::ProfileDownloader,
    PinnedFutureWithBounds,
};

use super::{vanilla::Vanilla, ToLoaderProfile};

pub struct VanillaCombinedDownloader<T> {
    version: String,
    game_paths: GamePaths,
    vanilla: Vanilla,
    loader: T,
}

impl<T> Debug for VanillaCombinedDownloader<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VanillaCombinedDownloader")
            .field("version", &self.version)
            .field("game_paths", &self.game_paths)
            .field("vanilla", &self.vanilla)
            .field("loader", &"(loader)")
            .finish()
    }
}

impl VanillaCombinedDownloader<()> {
    pub async fn new(game_version: impl Into<String>, game_paths: GamePaths) -> anyhow::Result<Self> {
        let version = game_version.into();
        let vanilla = Vanilla::new(&version, game_paths.clone()).await?;

        Ok(Self {
            version,
            game_paths,
            vanilla,
            loader: (),
        })
    }

    pub async fn with_loader<T, F, Fut>(self, fun: F) -> anyhow::Result<VanillaCombinedDownloader<T>>
    where
        F: FnOnce(String, GamePaths) -> Fut,
        Fut: Future<Output = anyhow::Result<T>>,
        T: ProfileDownloader,
    {
        let loader = (fun)(self.version.clone(), self.game_paths.clone()).await?;

        Ok(VanillaCombinedDownloader {
            version: self.version,
            game_paths: self.game_paths,
            vanilla: self.vanilla,
            loader,
        })
    }
}

impl<T: ToLoaderProfile> ToLoaderProfile for VanillaCombinedDownloader<T> {
    fn to_profile(&self) -> crate::instance::loader::LoaderProfile {
        self.loader.to_profile()
    }
}

#[async_trait::async_trait]
impl<T: ProfileDownloader + 'static> Downloader for VanillaCombinedDownloader<T> {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.vanilla.total() + self.loader.total()
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let downloader = DownloadQueue::new().with_downloader(self.vanilla).with_downloader(self.loader);
        let downloader = Box::new(downloader);
        downloader.download(sender).await;
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        let vanilla_io = self.vanilla.io();
        let loader_io = self.loader.io();

        Box::pin(async move {
            vanilla_io.await?;
            loader_io.await
        })
    }
}

#[async_trait::async_trait]
impl Downloader for VanillaCombinedDownloader<()> {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.vanilla.total()
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        Box::new(self.vanilla).download(sender).await;
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        self.vanilla.io()
    }
}

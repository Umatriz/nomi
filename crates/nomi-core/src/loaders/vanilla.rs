use std::path::Path;

use reqwest::Client;

use tracing::error;

use crate::{
    downloads::{
        downloaders::{
            file::FileDownloader,
            libraries::{LibrariesDownloader, LibrariesMapper},
        },
        progress::ProgressSender,
        traits::{DownloadResult, Downloader},
        DownloadQueue,
    },
    fs::write_to_file,
    game_paths::GamePaths,
    repository::manifest::{Classifiers, DownloadFile, Library, Manifest},
    state::get_launcher_manifest,
    PinnedFutureWithBounds,
};

#[derive(Debug)]
pub struct Vanilla {
    manifest: Manifest,
    game_paths: GamePaths,
    queue: DownloadQueue,
}

impl Vanilla {
    pub async fn new(version_id: impl Into<String>, game_paths: GamePaths) -> anyhow::Result<Self> {
        let id = version_id.into();
        let client = Client::new();
        let launcher_manifest = get_launcher_manifest().await?;

        let Some(val) = launcher_manifest.versions.iter().find(|i| i.id == id) else {
            error!("Cannot find this version");

            return Err(crate::error::Error::NoSuchVersion.into());
        };

        let manifest = client.get(&val.url).send().await?.json::<Manifest>().await?;

        let libraries_mapper = VanillaLibrariesMapper { path: &game_paths.libraries };

        let native_libraries_mapper = VanillaNativeLibrariesMapper { path: &game_paths.libraries };

        let queue = DownloadQueue::new()
            .with_downloader(LibrariesDownloader::new(&libraries_mapper, &manifest.libraries))
            .with_downloader(LibrariesDownloader::new(&native_libraries_mapper, &manifest.libraries))
            .with_downloader(
                FileDownloader::new(
                    manifest.downloads.client.url.clone(),
                    game_paths.profile.join(format!("{}.jar", manifest.id)),
                )
                .into_retry(),
            );

        Ok(Self { manifest, game_paths, queue })
    }
}

fn manifest_file_to_downloader(manifest_file: &DownloadFile, target_path: &Path) -> Option<FileDownloader> {
    manifest_file
        .path
        .as_ref()
        .map(|path| target_path.join(path))
        .map(|path| (manifest_file.url.clone(), path))
        .filter(|(_, path)| !path.exists())
        .map(|(url, path)| FileDownloader::new(url, path))
}

pub(crate) struct VanillaLibrariesMapper<'a> {
    pub path: &'a Path,
}

impl LibrariesMapper<Library> for VanillaLibrariesMapper<'_> {
    fn proceed(&self, library: &Library) -> Option<FileDownloader> {
        library
            .downloads
            .artifact
            .as_ref()
            .and_then(|file| manifest_file_to_downloader(file, self.path))
    }
}

struct VanillaNativeLibrariesMapper<'a> {
    path: &'a Path,
}

impl LibrariesMapper<Library> for VanillaNativeLibrariesMapper<'_> {
    fn proceed(&self, library: &Library) -> Option<FileDownloader> {
        fn match_natives(natives: &Classifiers) -> Option<&DownloadFile> {
            match std::env::consts::OS {
                "linux" => natives.natives_linux.as_ref(),
                "windows" => natives.natives_windows.as_ref(),
                "macos" => natives.natives_macos.as_ref(),
                _ => unreachable!(),
            }
        }

        library
            .downloads
            .classifiers
            .as_ref()
            .and_then(match_natives)
            .and_then(|file| manifest_file_to_downloader(file, self.path))
    }
}

#[async_trait::async_trait]
impl Downloader for Vanilla {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.queue.total()
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        Box::new(self.queue).download(sender).await;
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        let versions_path = self.game_paths.profile.clone();
        let manifest_id = self.manifest.id.clone();
        let manifest_res = serde_json::to_string_pretty(&self.manifest);

        let fut = async move {
            let path = versions_path.join(format!("{manifest_id}.json"));

            let body = manifest_res?;

            write_to_file(body.as_bytes(), &path).await
        };

        Box::pin(fut)
    }
}

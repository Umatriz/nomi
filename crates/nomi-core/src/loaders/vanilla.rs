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
        traits::{DownloadResult, Downloader, DownloaderIO, DownloaderIOExt},
        DownloadQueue,
    },
    fs::write_to_file,
    game_paths::GamePaths,
    repository::manifest::{Classifiers, DownloadFile, Library, Manifest},
    state::get_launcher_manifest,
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
            .with_downloader(FileDownloader::new(
                manifest.downloads.client.url.clone(),
                game_paths.version.join(format!("{}.jar", manifest.id)),
            ));

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

struct VanillaLibrariesMapper<'a> {
    path: &'a Path,
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

#[allow(clippy::module_name_repetitions)]
pub struct VanillaIO<'a> {
    manifest: &'a Manifest,
    version_path: &'a Path,
}

#[async_trait::async_trait]
impl DownloaderIO for VanillaIO<'_> {
    async fn io(&self) -> anyhow::Result<()> {
        let path = self.version_path.join(format!("{}.json", self.manifest.id));

        let body = serde_json::to_string_pretty(&self.manifest)?;

        write_to_file(body.as_bytes(), &path).await
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
}

impl<'a> DownloaderIOExt<'a> for Vanilla {
    type IO = VanillaIO<'a>;

    fn get_io(&'a self) -> VanillaIO<'a> {
        VanillaIO {
            manifest: &self.manifest,
            version_path: &self.game_paths.version,
        }
    }
}

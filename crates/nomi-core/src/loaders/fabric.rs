use std::path::{Path, PathBuf};

use reqwest::Client;
use tokio::sync::mpsc::Sender;

use crate::{
    downloads::{
        downloaders::{
            file::FileDownloader,
            libraries::{LibrariesDownloader, LibrariesMapper},
        },
        traits::{DownloadResult, Downloader, DownloaderIO, DownloaderIOExt},
    },
    fs::write_to_file,
    game_paths::GamePaths,
    maven_data::MavenData,
    repository::{
        fabric_meta::FabricVersions,
        fabric_profile::{FabricLibrary, FabricProfile},
    },
    state::get_launcher_manifest,
};

#[derive(Debug)]
pub struct Fabric {
    pub game_version: String,
    pub profile: FabricProfile,
    game_paths: GamePaths,
    libraries_downloader: LibrariesDownloader,
}

impl Fabric {
    pub async fn new(
        game_version: impl Into<String>,
        loader_version: Option<impl Into<String>>,
        game_paths: GamePaths,
    ) -> anyhow::Result<Self> {
        let game_version = game_version.into();

        let client = Client::new();
        let launcher_manifest = get_launcher_manifest().await?;

        if !launcher_manifest
            .versions
            .iter()
            .any(|v| v.id == game_version)
        {
            return Err(crate::error::Error::NoSuchVersion.into());
        };

        let versions: FabricVersions = client
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{game_version}"
            ))
            .send()
            .await?
            .json()
            .await?;

        let profile_version = loader_version
            .map(Into::into)
            .and_then(|loader| versions.iter().find(|i| i.loader.version == loader))
            .unwrap_or_else(|| &versions[0]);

        let profile: FabricProfile = client
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                game_version, profile_version.loader.version
            ))
            .send()
            .await?
            .json()
            .await?;

        let mapper = FabricLibrariesMapper {
            libraries: game_paths.libraries.clone(),
        };

        let libraries_downloader = LibrariesDownloader::new(&mapper, &profile.libraries);

        Ok(Self {
            game_version,
            profile,
            game_paths,
            libraries_downloader,
        })
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct FabricIO<'a> {
    profile: &'a FabricProfile,
    version_path: &'a Path,
}

#[async_trait::async_trait]
impl<'a> DownloaderIO for FabricIO<'a> {
    async fn io(&self) -> anyhow::Result<()> {
        let path = self.version_path.join(format!("{}.json", self.profile.id));

        let body = serde_json::to_string_pretty(&self.profile)?;

        write_to_file(body.as_bytes(), &path).await
    }
}

struct FabricLibrariesMapper {
    libraries: PathBuf,
}

impl LibrariesMapper<FabricLibrary> for FabricLibrariesMapper {
    fn proceed(&self, library: &FabricLibrary) -> Option<FileDownloader> {
        let data = MavenData::new(&library.name);
        let path = self.libraries.join(&data.path);

        (!path.exists()).then_some(FileDownloader::new(
            format!("{}{}", library.url, data.url),
            path,
        ))
    }
}

#[async_trait::async_trait]
impl Downloader for Fabric {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.libraries_downloader.total()
    }

    async fn download(self: Box<Self>, channel: Sender<Self::Data>) {
        Box::new(self.libraries_downloader).download(channel).await;
    }
}

impl<'a> DownloaderIOExt<'a> for Fabric {
    type IO = FabricIO<'a>;

    fn get_io(&'a self) -> FabricIO<'a> {
        FabricIO {
            profile: &self.profile,
            version_path: &self.game_paths.version,
        }
    }
}

// #[async_trait::async_trait]
// impl DownloadVersion for Fabric {
//     async fn download(&self, dir: &Path, file_name: &str) -> anyhow::Result<()> {
//         // Vanilla::new(&self.game_version)
//         //     .await?
//         //     .download(dir, file_name)
//         //     .await?;

//         info!("Fabric downloaded successfully");

//         Ok(())
//     }

//     async fn download_libraries(&self, dir: &Path) -> anyhow::Result<()> {
//         let mut set = JoinSet::new();

//         self.profile.libraries.iter().for_each(|lib| {
//             let maven = MavenData::new(&lib.name);
//             let path = dir.join(maven.path);
//             if !path.exists() {
//                 set.spawn(download_file(path, format!("{}{}", lib.url, maven.url)));
//             }
//         });

//         while let Some(res) = set.join_next().await {
//             res??
//         }

//         Ok(())
//     }

//     async fn create_json(&self, dir: &Path) -> anyhow::Result<()> {
//         let file_name = format!("{}.json", self.profile.id);
//         let path = dir.join(file_name);

//         let body = serde_json::to_string_pretty(&self.profile)?;

//         write_to_file(body.as_bytes(), &path).await?;

//         info!(
//             "Version json {} created successfully",
//             path.to_string_lossy()
//         );

//         Ok(())
//     }
// }

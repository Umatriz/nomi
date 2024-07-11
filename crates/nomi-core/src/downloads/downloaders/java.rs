use std::{
    fs::File,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::error;

use crate::{downloads::progress::ProgressSender, DOT_NOMI_TEMP_DIR};

use super::{
    super::traits::{DownloadResult, Downloader, DownloaderIO, DownloaderIOExt},
    FileDownloader,
};

#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
mod consts {
    pub(super) const PORTABLE_URL: &str =
        "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_windows-x64_bin.zip";
    pub(super) const SHA256: &str = "de7f00fd1bd0d3a4c678fff2681dfad19284d74d357218a4be6f623488d040da";
    pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_windows-x64_bin.zip";
}

#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
mod consts {
    pub(super) const PORTABLE_URL: &str =
        "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_macos-x64_bin.tar.gz";
    pub(super) const SHA256: &str = "5daa4f9894cc3a617a5f9fe2c48e5391d3a2e672c91e1597041672f57696846f";
    pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_macos-x64_bin.tar.gz";
}

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod consts {
    pub(super) const PORTABLE_URL: &str =
        "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_macos-aarch64_bin.tar.gz";
    pub(super) const SHA256: &str = "b949a3bc13e3c5152ab55d12e699dfa6c8b00bedeb8302b13be4aec3ee734351";
    pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_macos-aarch64_bin.tar.gz";
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod consts {
    pub(super) const PORTABLE_URL: &str =
        "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_linux-x64_bin.tar.gz";
    pub(super) const SHA256: &str = "133c8b65113304904cdef7c9103274d141cfb64b191ff48ceb6528aca25c67b1";
    pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_linux-x64_bin.tar.gz";
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
mod consts {
    pub(super) const PORTABLE_URL: &str =
        "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_linux-aarch64_bin.tar.gz";
    pub(super) const SHA256: &str = "0887c42b9897f889415a6f7b88549d38af99f6ef2d1117199de012beab0631eb";
    pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_linux-aarch64_bin.tar.gz";
}

fn check_hash(path: PathBuf, hash: &str) -> anyhow::Result<bool> {
    let sha = sha256::try_digest(path)?;
    Ok(dbg!(sha) == dbg!(hash))
}

/// You must call `JavaDownloaderIO::io` in order to see a result
pub struct JavaDownloader {
    target_directory: PathBuf,
}

impl JavaDownloader {
    pub fn new(target_directory: PathBuf) -> Self {
        Self { target_directory }
    }
}

#[async_trait::async_trait]
impl Downloader for JavaDownloader {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        1
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let downloader = FileDownloader::new(
            consts::PORTABLE_URL.to_string(),
            PathBuf::from(DOT_NOMI_TEMP_DIR).join(consts::ARCHIVE_FILENAME),
        );

        Box::new(downloader).download(sender).await;
    }
}

impl<'a> DownloaderIOExt<'a> for JavaDownloader {
    type IO = JavaDownloaderIO;

    fn get_io(&'a self) -> Self::IO {
        JavaDownloaderIO {
            target_directory: self.target_directory.clone(),
        }
    }
}

pub struct JavaDownloaderIO {
    target_directory: PathBuf,
}

#[cfg(target_os = "windows")]
fn extract(archive: std::fs::File, target_path: &Path) -> anyhow::Result<()> {
    let mut zip = zip::ZipArchive::new(archive)?;
    zip.extract(target_path).map_err(Into::into)
}

#[cfg(not(target_os = "windows"))]
fn extract(archive: std::fs::File, target_path: &Path) -> anyhow::Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tar = GzDecoder::new(archive);
    let mut archive = Archive::new(tar);
    archive.unpack(target_path).map_err(Into::into)
}

#[async_trait::async_trait]
impl DownloaderIO for JavaDownloaderIO {
    async fn io(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(DOT_NOMI_TEMP_DIR).join(consts::ARCHIVE_FILENAME);
        if !check_hash(path.clone(), consts::SHA256)? {
            return Err(JavaDownloaderError::HashDoesNotMatch.into());
        }

        let file = File::open(&path)?;

        extract(file, &self.target_directory)?;

        tokio::fs::remove_file(&path).await?;

        Ok(())
    }
}

#[derive(Error, Debug)]
enum JavaDownloaderError {
    #[error("Hash does not match")]
    HashDoesNotMatch,

    #[error("data store disconnected")]
    IoError(#[from] std::io::Error),

    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Join error")]
    JoinError(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod consts0 {
        pub(super) const PORTABLE_URL: &str =
            "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_windows-x64_bin.zip";
        pub(super) const SHA256: &str = "de7f00fd1bd0d3a4c678fff2681dfad19284d74d357218a4be6f623488d040da";
        pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_windows-x64_bin.zip";
    }

    mod consts1 {
        pub(super) const PORTABLE_URL: &str =
            "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_macos-x64_bin.tar.gz";
        pub(super) const SHA256: &str = "5daa4f9894cc3a617a5f9fe2c48e5391d3a2e672c91e1597041672f57696846f";
        pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_macos-x64_bin.tar.gz";
    }

    mod consts2 {
        pub(super) const PORTABLE_URL: &str =
            "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_macos-aarch64_bin.tar.gz";
        pub(super) const SHA256: &str = "b949a3bc13e3c5152ab55d12e699dfa6c8b00bedeb8302b13be4aec3ee734351";
        pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_macos-aarch64_bin.tar.gz";
    }

    mod consts3 {
        pub(super) const PORTABLE_URL: &str =
            "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_linux-x64_bin.tar.gz";
        pub(super) const SHA256: &str = "133c8b65113304904cdef7c9103274d141cfb64b191ff48ceb6528aca25c67b1";
        pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_linux-x64_bin.tar.gz";
    }

    mod consts4 {
        pub(super) const PORTABLE_URL: &str =
            "https://download.java.net/java/GA/jdk22.0.1/c7ec1332f7bb44aeba2eb341ae18aca4/8/GPL/openjdk-22.0.1_linux-aarch64_bin.tar.gz";
        pub(super) const SHA256: &str = "0887c42b9897f889415a6f7b88549d38af99f6ef2d1117199de012beab0631eb";
        pub(super) const ARCHIVE_FILENAME: &str = "openjdk-22.0.1_linux-aarch64_bin.tar.gz";
    }

    async fn java_downloader_test_helper(url: &str, file_name: &str, hash: &str) -> anyhow::Result<bool> {
        let downloader = FileDownloader::new(url.to_owned(), PathBuf::from("./java_downloader_test").join(file_name));

        let (tx, mut rx) = tokio::sync::mpsc::channel(5);

        Box::new(downloader).download(&tx).await;

        dbg!(rx.recv().await);

        check_hash(PathBuf::from("./java_downloader_test").join(file_name), hash)
    }

    #[tokio::test]
    async fn java_downloader_test() {
        macro_rules! java_downloader_test {
            (
                $($ident:ident)*
            ) => {
                $(
                    assert!(java_downloader_test_helper(
                        $ident::PORTABLE_URL,
                        $ident::ARCHIVE_FILENAME,
                        $ident::SHA256
                    )
                    .await
                    .unwrap());
                )*
            };
        }

        java_downloader_test! {
            consts0 consts1 consts2 consts3 consts4
        }

        tokio::fs::remove_dir_all("./java_downloader_test").await.unwrap();
    }

    #[tokio::test]
    async fn tarball_structure_test() {
        fn extract_tarball(archive: std::fs::File, target_path: &Path) -> anyhow::Result<()> {
            use flate2::read::GzDecoder;
            use tar::Archive;

            let tar = GzDecoder::new(archive);
            let mut archive = Archive::new(tar);
            archive.unpack(target_path).map_err(Into::into)
        }

        let downloader = FileDownloader::new(consts3::PORTABLE_URL.to_owned(), PathBuf::from("./").join(consts3::ARCHIVE_FILENAME));

        let (tx, mut rx) = tokio::sync::mpsc::channel(5);

        Box::new(downloader).download(&tx).await;

        dbg!(rx.recv().await);

        if !check_hash(PathBuf::from("./").join(consts3::ARCHIVE_FILENAME), consts3::SHA256).unwrap() {
            panic!("Hashes does not match");
        }

        let file = File::open(consts3::ARCHIVE_FILENAME).unwrap();

        extract_tarball(file, &PathBuf::from("./java_test")).unwrap();

        tokio::fs::remove_file(PathBuf::from("./").join(consts3::ARCHIVE_FILENAME)).await.unwrap();

        tokio::fs::remove_dir_all(PathBuf::from("./java_test")).await.unwrap();
    }
}

use std::path::{Path, PathBuf};
use thiserror::Error;

use super::download_file;

const PORTABLE_URL: &str = "https://download.oracle.com/java/17/latest/jdk-17_windows-x64_bin.zip";

const JDK_17_0_7_PORTABLE_SHA256: &str =
    "98385c1fd4db7ad3fd7ca2f33a1fadae0b15486cfde699138d47002d7068084a";

async fn check_hash(path: PathBuf, hash: &str) -> anyhow::Result<()> {
    let sha = sha256::try_digest(path)?;

    if sha != hash {
        return Err(JavaInstallerError::HashDoesNotMatch.into());
    };
    Ok(())
}

pub async fn download_java(temporary_dir_path: &Path, java_dir_path: &Path) -> anyhow::Result<()> {
    let archive_filename = "jdk-17_windows-x64_bin.zip";
    download_file(
        temporary_dir_path.join(archive_filename),
        PORTABLE_URL.to_string(),
    )
    .await?;

    check_hash(
        temporary_dir_path.join(archive_filename),
        JDK_17_0_7_PORTABLE_SHA256,
    )
    .await?;

    let archive = std::fs::File::open(temporary_dir_path.join(archive_filename))?;

    let mut zip = zip::ZipArchive::new(archive)?;

    zip.extract(java_dir_path)?;

    if let Err(err) = std::fs::remove_file(temporary_dir_path.join(archive_filename)) {
        log::error!("Error occurred during file removing\nError: {}", err);
    }

    Ok(())
}

#[derive(Error, Debug)]
enum JavaInstallerError {
    #[error("Hash does not match")]
    HashDoesNotMatch,

    #[error("data store disconnected")]
    IoError(#[from] std::io::Error),

    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Join error")]
    JoinError(#[from] tokio::task::JoinError),
}

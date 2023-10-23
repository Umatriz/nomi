use std::path::Path;
use thiserror::Error;
use tracing::error;

use crate::utils::download_util::{download_file, check_sha256};
use crate::configs::consts::{PORTABLE_URL, JDK_17_0_7_PORTABLE_SHA256};

pub async fn download_java(temporary_dir_path: &Path, java_dir_path: &Path) -> anyhow::Result<()> {
    let archive_filename = "jdk-17_windows-x64_bin.zip";
    download_file(
        temporary_dir_path.join(archive_filename),
        PORTABLE_URL.to_string(),
    )
    .await?;

    check_sha256(
        temporary_dir_path.join(archive_filename),
        JDK_17_0_7_PORTABLE_SHA256,
    )
    .await?;

    let archive = std::fs::File::open(temporary_dir_path.join(archive_filename))?;

    let mut zip = zip::ZipArchive::new(archive)?;

    zip.extract(java_dir_path)?;

    if let Err(err) = std::fs::remove_file(temporary_dir_path.join(archive_filename)) {
        error!("Error occurred during file removing\nError: {}", err);
    }

    Ok(())
}

#[derive(Error, Debug)]
enum JavaInstallerError {

    #[error("Data store disconnected")]
    IoError(#[from] std::io::Error),

    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Join error")]
    JoinError(#[from] tokio::task::JoinError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        download_java(Path::new("./"), Path::new("java"))
            .await
            .unwrap();
    }
}

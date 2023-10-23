use futures_util::stream::StreamExt;
use reqwest::Client;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, trace};

#[derive(Error, Debug)]
enum DownloadManagerError {
    #[error("Hash does not match")]
    HashDoesNotMatch,
}

pub(crate) async fn check_sha256(path: PathBuf, hash: &str) -> anyhow::Result<()> {
    let sha = sha256::try_digest(path)?;

    if sha != hash {
        return Err(DownloadManagerError::HashDoesNotMatch.into());
    };
    Ok(())
}

pub(crate) async fn download_file<P: AsRef<Path>>(
    path: P,
    url: impl Into<String>,
) -> anyhow::Result<()> {
    let path = path.as_ref();

    if let Some(path) = path.parent() {
        tokio::fs::create_dir_all(path).await?;
    }

    let client = Client::new();
    let res = client.get(&url.into()).send().await?;

    let mut file = tokio::fs::File::create(path).await.map_err(|err| {
        error!(
            "Error occurred during file creating\nPath: {}\nError: {}",
            path.to_string_lossy(),
            err
        );
        err
    })?;

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|err| {
            error!("Error occurred during file downloading\nError: {}", err);
            err
        })?;

        file.write_all(&chunk).await.map_err(|err| {
            error!("Error occurred during writing to file\nError: {}", err);
            err
        })?;
    }

    debug!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}

pub(crate) async fn create_dir(main_dir: &Path, dir_name: &str) -> anyhow::Result<PathBuf> {
    let path = main_dir.join(dir_name);

    tokio::fs::create_dir_all(&path).await?;

    trace!("Dir {} created successfully", path.to_string_lossy());

    Ok(path)
}
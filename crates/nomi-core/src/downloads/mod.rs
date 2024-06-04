pub use downloaders::*;

use std::path::{Path, PathBuf};

use futures_util::stream::StreamExt;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, task::JoinError};
use tracing::{debug, error};

pub mod downloaders;
pub mod traits;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("DownloadError:\nurl: {url}\npath: {path}\nerror: {error:#?}")]
    Error {
        url: String,
        path: PathBuf,
        error: String,
    },

    #[error("{0}")]
    JoinError(#[from] JoinError),
}

pub(crate) async fn download_file(
    path: impl AsRef<Path>,
    url: impl Into<String>,
) -> Result<(), DownloadError> {
    let url = url.into();
    let path = path.as_ref();

    if let Some(path) = path.parent() {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|err| DownloadError::Error {
                url: url.clone(),
                path: path.to_path_buf(),
                error: err.to_string(),
            })?;
    }

    let client = Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|err| DownloadError::Error {
            url: url.clone(),
            path: path.to_path_buf(),
            error: err.to_string(),
        })?;

    let mut file = tokio::fs::File::create(path).await.map_err(|err| {
        error!(
            "Error occurred during file creating\nPath: {}\nError: {}",
            path.to_string_lossy(),
            err
        );
        DownloadError::Error {
            url: url.clone(),
            path: path.to_path_buf(),
            error: err.to_string(),
        }
    })?;

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|err| {
            error!("Error occurred during file downloading\nError: {}", err);
            DownloadError::Error {
                url: url.clone(),
                path: path.to_path_buf(),
                error: err.to_string(),
            }
        })?;

        file.write_all(&chunk).await.map_err(|err| {
            error!("Error occurred during writing to file\nError: {}", err);
            DownloadError::Error {
                url: url.clone(),
                path: path.to_path_buf(),
                error: err.to_string(),
            }
        })?;
    }

    debug!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}

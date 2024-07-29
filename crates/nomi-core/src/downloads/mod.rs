pub use downloaders::*;

use std::{
    future::IntoFuture,
    path::{Path, PathBuf},
};

use futures_util::stream::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use tokio::io::AsyncWriteExt;
use tracing::{error, trace};

use crate::PinnedFutureWithBounds;

pub mod downloaders;
pub mod progress;
pub mod traits;

#[derive(Debug, thiserror::Error, Clone)]
pub enum DownloadError {
    #[error("DownloadError:\nurl: {url}\npath: {path}\nerror: {error:#?}")]
    Error { url: String, path: PathBuf, error: String },

    #[error("Hashes does not match.\nurl: {url}\npath: {path}\nerror: {error:#?}")]
    HashDoesNotMatch {
        url: String,
        path: PathBuf,
        sha1: String,
        error: String,
    },

    #[error("The task was cancelled or panicked")]
    JoinError,

    #[error("All iterations failed")]
    AllIterationsFailed,
}

pub(crate) fn download_file(path: impl AsRef<Path>, url: impl Into<String>) -> Downloader {
    let url = url.into();
    let path = path.as_ref().to_path_buf();

    Downloader {
        url,
        path,
        client: None,
        request_injection: Box::new(|r| r),
    }
}

pub struct Downloader {
    url: String,
    path: PathBuf,
    client: Option<Client>,
    request_injection: Box<dyn FnOnce(RequestBuilder) -> RequestBuilder + Send>,
}

impl Downloader {
    #[must_use]
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    #[must_use]
    pub fn with_request_injection(mut self, injection: impl FnOnce(RequestBuilder) -> RequestBuilder + Send + 'static) -> Self {
        self.request_injection = Box::new(injection);
        self
    }

    async fn download(self) -> Result<(), DownloadError> {
        let download_error = |error| -> DownloadError {
            DownloadError::Error {
                url: self.url.clone(),
                path: self.path.clone(),
                error,
            }
        };

        if let Some(path) = self.path.parent() {
            tokio::fs::create_dir_all(path).await.map_err(|err| download_error(err.to_string()))?;
        }

        let client = self.client.unwrap_or_default();
        let request = (self.request_injection)(client.get(&self.url));
        let res = request
            .send()
            .await
            .and_then(Response::error_for_status)
            .map_err(|err| download_error(err.to_string()))?;

        let mut file = tokio::fs::File::create(&self.path).await.map_err(|err| {
            error!(
                "Error occurred during file creating\nPath: {}\nError: {}",
                self.path.to_string_lossy(),
                err
            );
            download_error(err.to_string())
        })?;

        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|err| {
                error!("Error occurred during file downloading\nError: {}", err);
                download_error(err.to_string())
            })?;

            file.write_all(&chunk).await.map_err(|err| {
                error!("Error occurred during writing to file\nError: {}", err);
                download_error(err.to_string())
            })?;
        }

        trace!("Downloaded successfully {}", self.path.to_string_lossy());

        Ok(())
    }
}

impl IntoFuture for Downloader {
    type Output = Result<(), DownloadError>;

    type IntoFuture = PinnedFutureWithBounds<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.download())
    }
}

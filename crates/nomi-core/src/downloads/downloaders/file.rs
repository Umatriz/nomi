use std::path::PathBuf;

use sha1::Digest;
use tracing::{debug, error};

use crate::{
    calculate_sha1,
    downloads::{
        download_file,
        traits::{DownloadResult, DownloadStatus, Downloadable},
        DownloadError,
    },
};

#[derive(Debug)]
pub struct FileDownloader {
    url: String,
    path: PathBuf,
    hash_sha1: Option<String>,
}

impl FileDownloader {
    pub fn new(url: String, path: PathBuf) -> Self {
        Self {
            url,
            path,
            hash_sha1: None,
        }
    }

    #[must_use]
    pub fn with_sha1(mut self, hash: String) -> Self {
        self.hash_sha1 = Some(hash);
        self
    }
}

#[async_trait::async_trait]
impl Downloadable for FileDownloader {
    type Out = DownloadResult;

    #[tracing::instrument(name = "File download", res(level = Level::Trace))]
    #[allow(clippy::blocks_in_conditions)]
    async fn download(self: Box<Self>) -> Self::Out {
        let result = download_file(&self.path, &self.url)
            .await
            .map(|()| DownloadStatus::Success);

        let Ok(_) = result else {
            return DownloadResult(result);
        };

        if let Some(hash) = self.hash_sha1 {
            let file = match tokio::fs::read_to_string(&self.path).await.map_err(|e| {
                DownloadError::Error {
                    url: self.url.clone(),
                    path: self.path.clone(),
                    error: format!("Unable to open downloaded file. Original error: {e}"),
                }
            }) {
                Ok(val) => val,
                Err(e) => return DownloadResult(result.map_err(|_| e)),
            };

            let calculated_hash = calculate_sha1(file);

            if hash == calculated_hash {
                debug!("Hashes matched successfully");
            } else {
                let s = format!("Hashes does not match. {hash} != {calculated_hash}");
                error!("{s}");
                return DownloadResult(Err(DownloadError::Error {
                    url: self.url.clone(),
                    path: self.path.clone(),
                    error: s,
                }));
            }
        }

        DownloadResult(result)
    }
}

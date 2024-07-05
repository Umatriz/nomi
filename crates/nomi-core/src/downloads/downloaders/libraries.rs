use itertools::Itertools;

use crate::downloads::{
    progress::ProgressSender,
    traits::{DownloadResult, Downloader},
    DownloadSet,
};

use super::file::FileDownloader;

pub trait LibrariesMapper<L> {
    fn proceed(&self, library: &L) -> Option<FileDownloader>;
}

#[derive(Debug)]
pub struct LibrariesDownloader {
    downloads: Vec<FileDownloader>,
}

impl LibrariesDownloader {
    pub fn new<M, L>(mapper: &M, libraries: &[L]) -> Self
    where
        M: LibrariesMapper<L>,
    {
        let downloads = libraries
            .iter()
            .filter_map(|lib| mapper.proceed(lib))
            .collect_vec();

        Self { downloads }
    }
}

#[async_trait::async_trait]
impl Downloader for LibrariesDownloader {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.downloads.len() as u32
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let mut download_set = DownloadSet::new();

        for downloader in self.downloads {
            download_set.add(Box::new(downloader));
        }

        Box::new(download_set).download(sender).await;
    }
}

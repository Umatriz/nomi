use std::{fmt::Debug, time::Duration};

use dyn_clone::DynClone;
use tracing::{error, warn};

use crate::downloads::{
    traits::{DownloadResult, Downloadable},
    DownloadError,
};

pub struct ReTryDownloader {
    downloadable: Box<dyn DynCloneDownloadable>,
    iterations: usize,
    duration: Duration,
}

impl Debug for ReTryDownloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReTry")
            .field("downloadable", &"(Downloadable)")
            .field("iterations", &self.iterations)
            .field("duration", &self.duration)
            .finish()
    }
}

trait DynCloneDownloadable: Downloadable<Out = DownloadResult> + DynClone {}

impl<T> DynCloneDownloadable for T where T: Downloadable<Out = DownloadResult> + DynClone {}

impl ReTryDownloader {
    /// Create a new retry with the number of iterations of 5.
    pub fn new<D>(downloadable: D) -> Self
    where
        D: Downloadable<Out = DownloadResult> + DynClone + 'static,
    {
        Self {
            downloadable: Box::new(downloadable),
            iterations: 5,
            duration: Duration::from_secs(3),
        }
    }

    #[must_use]
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    #[must_use]
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

#[async_trait::async_trait]
impl Downloadable for ReTryDownloader {
    type Out = DownloadResult;

    #[tracing::instrument(skip(self), fields(iterations = self.iterations, time_between = tracing::field::debug(&self.duration)))]
    async fn download(self: Box<Self>) -> Self::Out {
        for i in 0..=self.iterations {
            let downloadable = dyn_clone::clone_box(&*self.downloadable);
            match downloadable.download().await.0 {
                Ok(ok) => return DownloadResult(Ok(ok.clone())),
                Err(err) => warn!("Downloading iteration {i} failed. Retrying. Error: {err}"),
            }
            // Wait between iterations
            tokio::time::sleep(self.duration).await;
        }

        error!("All iterations failed");

        DownloadResult(Err(DownloadError::AllIterationsFailed))
    }
}

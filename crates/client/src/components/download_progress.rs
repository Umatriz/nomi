use std::sync::Arc;

use eframe::egui;
use nomi_core::{configs::profile::VersionProfile, downloads::traits::DownloadResult};
use tokio::sync::mpsc::Receiver;

use crate::Storage;

use super::{profiles::ProfilesData, Component, StorageCreationExt};

pub struct DownloadProgress<'a> {
    pub storage: &'a mut Storage,

    pub download_result_rx: &'a mut Receiver<VersionProfile>,
    pub download_progress_rx: &'a mut Receiver<DownloadResult>,
    pub download_total_rx: &'a mut Receiver<u32>,
}

struct DownloadProgressData {
    total: u32,
    current: u32,
}

impl StorageCreationExt for DownloadProgress<'_> {
    fn extend(storage: &mut Storage) -> anyhow::Result<()> {
        storage.insert(DownloadProgressData {
            total: 0,
            current: 0,
        });
        Ok(())
    }
}

impl Component for DownloadProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        {
            let profiles = self.storage.get_mut::<ProfilesData>().unwrap();

            if let Ok(profile) = self.download_result_rx.try_recv() {
                let prof = profiles
                    .profiles
                    .iter_mut()
                    .find(|prof| prof.id == profile.id)
                    .unwrap();

                *prof = Arc::new(profile);
                profiles.update_config().unwrap();
            }
        }

        let progress_data = self.storage.get_mut::<DownloadProgressData>().unwrap();

        if let Ok(total) = self.download_total_rx.try_recv() {
            progress_data.total = total;
        }

        if let Ok(data) = self.download_progress_rx.try_recv() {
            progress_data.current += data.map_or(0, |_| 1);
        }

        ui.add(
            egui::ProgressBar::new(progress_data.current as f32 / progress_data.total as f32)
                .text(format!(
                    "{}/{} downloaded",
                    progress_data.current, progress_data.total
                ))
                .animate(true),
        );
    }
}

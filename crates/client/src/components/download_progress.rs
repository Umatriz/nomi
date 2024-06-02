use std::sync::Arc;

use eframe::egui;
use nomi_core::{configs::profile::VersionProfile, downloads::traits::DownloadResult};
use tokio::sync::mpsc::Receiver;

use super::{profiles::ProfilesState, Component};

pub struct DownloadProgress<'a> {
    pub download_progress_state: &'a mut DownloadProgressState,
    pub profiles_state: &'a mut ProfilesState,

    pub download_result_rx: &'a mut Receiver<VersionProfile>,
    pub download_progress_rx: &'a mut Receiver<DownloadResult>,
    pub download_total_rx: &'a mut Receiver<u32>,
}

#[derive(Default)]
pub struct DownloadProgressState {
    total: u32,
    current: u32,
}

impl Component for DownloadProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        {
            if let Ok(profile) = self.download_result_rx.try_recv() {
                let prof = self
                    .profiles_state
                    .profiles
                    .iter_mut()
                    .find(|prof| prof.id == profile.id)
                    .unwrap();

                *prof = Arc::new(profile);
                self.profiles_state.update_config().unwrap();
            }
        }

        if let Ok(total) = self.download_total_rx.try_recv() {
            self.download_progress_state.total = total;
            self.download_progress_state.current = 0;
        }

        if let Ok(data) = self.download_progress_rx.try_recv() {
            self.download_progress_state.current += data.map_or(0, |_| 1);
        }

        ui.add(
            egui::ProgressBar::new(
                self.download_progress_state.current as f32
                    / self.download_progress_state.total as f32,
            )
            .text(format!(
                "{}/{} downloaded",
                self.download_progress_state.current, self.download_progress_state.total
            ))
            .animate(true),
        );
    }
}

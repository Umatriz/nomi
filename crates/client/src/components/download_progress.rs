use std::{collections::HashMap, path::PathBuf, sync::Arc};

use eframe::egui;
use nomi_core::{configs::profile::VersionProfile, downloads::traits::DownloadResult};

use crate::{channel::Channel, download::spawn_assets, errors_pool::ErrorPoolExt};

use super::{profiles::ProfilesState, Component};

pub struct DownloadProgress<'a> {
    pub download_progress_state: &'a mut DownloadProgressState,
    pub profiles_state: &'a mut ProfilesState,
}

#[derive(Default)]
pub struct DownloadProgressState {
    pub assets_task: Option<Task<(), AssetsExtra>>,
    pub assets_to_download: Vec<Task<(), AssetsExtra>>,
    pub tasks: HashMap<u32, Task<VersionProfile>>,
}

pub struct AssetsExtra {
    pub version: String,
    pub assets_dir: PathBuf,
}

pub struct Task<R, Extra = ()> {
    name: String,
    total: u32,
    current: u32,
    is_finished: bool,
    extra: Extra,
    download_result_channel: Channel<R>,
    download_progress_channel: Channel<DownloadResult>,
    download_total_channel: Channel<u32>,
}

impl<R> Task<R> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            total: 0,
            current: 0,
            is_finished: false,
            extra: (),
            download_result_channel: Channel::new(100),
            download_progress_channel: Channel::new(500),
            download_total_channel: Channel::new(100),
        }
    }

    pub fn with_extra<Extra>(self, extra: Extra) -> Task<R, Extra> {
        Task {
            extra,
            name: self.name,
            total: self.total,
            current: self.current,
            is_finished: self.is_finished,
            download_result_channel: self.download_result_channel,
            download_progress_channel: self.download_progress_channel,
            download_total_channel: self.download_total_channel,
        }
    }
}

impl<R, Extra> Task<R, Extra> {
    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn result_channel(&self) -> &Channel<R> {
        &self.download_result_channel
    }
    pub fn progress_channel(&self) -> &Channel<DownloadResult> {
        &self.download_progress_channel
    }
    pub fn total_channel(&self) -> &Channel<u32> {
        &self.download_total_channel
    }

    pub fn result_channel_mut(&mut self) -> &mut Channel<R> {
        &mut self.download_result_channel
    }
    pub fn progress_channel_mut(&mut self) -> &mut Channel<DownloadResult> {
        &mut self.download_progress_channel
    }
    pub fn total_channel_mut(&mut self) -> &mut Channel<u32> {
        &mut self.download_total_channel
    }
}

impl Component for DownloadProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        if let Some(task) = &mut self.download_progress_state.assets_task {
            if task.result_channel_mut().try_recv().is_ok() {
                task.is_finished = true;
            }

            ui.label(format!("{0}", task.is_finished));

            if let Ok(total) = task.total_channel_mut().try_recv() {
                task.total = total;
                task.current = 0;
            }

            if let Ok(data) = task.progress_channel_mut().try_recv() {
                task.current += data.map_or(0, |_| 1);
            }

            ui.label(format!("Name: {}", task.name));
            if task.current != task.total {
                ui.add(
                    egui::ProgressBar::new(task.current as f32 / task.total as f32).animate(true),
                );
                ui.label(format!("{}/{} downloaded", task.current, task.total));
            } else {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Waiting for the progress data...");
                });
            }
        }

        if self
            .download_progress_state
            .assets_task
            .as_ref()
            .is_some_and(|task| task.is_finished)
        {
            self.download_progress_state.assets_task = None
        }

        if self.download_progress_state.assets_task.is_none() {
            if let Some(task) = self.download_progress_state.assets_to_download.pop() {
                spawn_assets(
                    task.extra().version.clone(),
                    task.extra().assets_dir.clone(),
                    task.result_channel().clone_tx(),
                    task.progress_channel().clone_tx(),
                    task.total_channel().clone_tx(),
                );
                self.download_progress_state.assets_task = Some(task);
            }
        }

        for task in self.download_progress_state.tasks.values_mut() {
            {
                if let Ok(profile) = task.result_channel_mut().try_recv() {
                    // PANICS: It will never panic because the profile
                    // cannot be downloaded if it doesn't exists
                    let prof = self
                        .profiles_state
                        .profiles
                        .iter_mut()
                        .find(|prof| prof.id == profile.id)
                        .unwrap();

                    *prof = Arc::new(profile);
                    self.profiles_state.update_config().report_error();

                    task.is_finished = true;
                }
            }

            if let Ok(total) = task.total_channel_mut().try_recv() {
                task.total = total;
                task.current = 0;
            }

            if let Ok(data) = task.progress_channel_mut().try_recv() {
                task.current += data.map_or(0, |_| 1);
            }

            ui.label(format!("Name: {}", task.name));
            if task.current != task.total {
                ui.add(
                    egui::ProgressBar::new(task.current as f32 / task.total as f32).animate(true),
                );
                ui.label(format!("{}/{} downloaded", task.current, task.total));
            } else {
                ui.label("Nothing to download");
            }
        }

        for (id, state) in self
            .download_progress_state
            .tasks
            .iter()
            .map(|(id, task)| (*id, task.is_finished))
            .filter(|(_, state)| *state)
            .collect::<Vec<_>>()
        {
            if state {
                self.download_progress_state.tasks.remove(&id);
            }
        }
    }
}

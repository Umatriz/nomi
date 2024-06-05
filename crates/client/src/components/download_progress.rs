use std::{collections::HashMap, path::PathBuf, sync::Arc};

use eframe::egui;
use nomi_core::{configs::profile::VersionProfile, downloads::traits::DownloadResult};
use tokio::task::JoinHandle;

use crate::{channel::Channel, download::spawn_assets, errors_pool::ErrorPoolExt};

use super::{profiles::ProfilesState, Component};

pub struct DownloadProgress<'a> {
    pub download_progress_state: &'a mut DownloadProgressState,
    pub profiles_state: &'a mut ProfilesState,
}

pub struct DownloadProgressState {
    pub is_allowed_to_take_action: bool,

    pub java_downloading_task: Option<Task<()>>,

    pub assets_task: Option<Task<(), AssetsExtra>>,
    pub assets_to_download: Vec<Task<(), AssetsExtra>>,
    pub profile_tasks: HashMap<u32, Task<VersionProfile>>,
}

impl Default for DownloadProgressState {
    fn default() -> Self {
        Self {
            is_allowed_to_take_action: true,
            java_downloading_task: None,
            assets_task: None,
            assets_to_download: Vec::new(),
            profile_tasks: HashMap::new(),
        }
    }
}

pub struct AssetsExtra {
    pub version: String,
    pub assets_dir: PathBuf,
}

pub struct Task<R, Extra = ()> {
    name: String,
    handle: Option<JoinHandle<()>>,
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
            handle: None,
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
            handle: self.handle,
            is_finished: self.is_finished,
            download_result_channel: self.download_result_channel,
            download_progress_channel: self.download_progress_channel,
            download_total_channel: self.download_total_channel,
        }
    }
}

impl<R, Extra> Task<R, Extra> {
    pub fn with_handle(mut self, handle: JoinHandle<()>) -> Self {
        self.handle = Some(handle);
        self
    }

    pub fn set_handle(&mut self, handle: JoinHandle<()>) {
        self.handle = Some(handle)
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn mark_finished(&mut self) {
        self.is_finished = true;
    }

    /// Can be used when you need to restart the task
    /// or if you are using the same task multiple times
    pub fn mark_unfinished(&mut self) {
        self.is_finished = false;
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished
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
        if let Some(task) = self.download_progress_state.java_downloading_task.as_mut() {
            show_task(ui, task, |_| ())
        }

        if self
            .download_progress_state
            .java_downloading_task
            .as_ref()
            .is_some_and(|task| task.is_finished)
        {
            self.download_progress_state.java_downloading_task = None
        }

        if let Some(task) = self.download_progress_state.assets_task.as_mut() {
            show_task(ui, task, |_| ())
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
                let handle = spawn_assets(
                    task.extra().version.clone(),
                    task.extra().assets_dir.clone(),
                    task.result_channel().clone_tx(),
                    task.progress_channel().clone_tx(),
                    task.total_channel().clone_tx(),
                );
                self.download_progress_state.assets_task = Some(task.with_handle(handle));
            }
        }

        for task in self.download_progress_state.profile_tasks.values_mut() {
            show_task(ui, task, |profile| {
                profile_callback(profile, self.profiles_state)
            });
        }

        for (id, state) in self
            .download_progress_state
            .profile_tasks
            .iter()
            .map(|(id, task)| (*id, task.is_finished))
            .filter(|(_, state)| *state)
            .collect::<Vec<_>>()
        {
            if state {
                self.download_progress_state.profile_tasks.remove(&id);
            }
        }
    }
}

fn profile_callback(profile: VersionProfile, profile_state: &mut ProfilesState) {
    // PANICS: It will never panic because the profile
    // cannot be downloaded if it doesn't exists
    let prof = profile_state
        .profiles
        .iter_mut()
        .find(|prof| prof.id == profile.id)
        .unwrap();

    *prof = Arc::new(profile);
    profile_state.update_config().report_error();
}

fn show_task<T, Extra>(
    ui: &mut egui::Ui,
    task: &mut Task<T, Extra>,
    mut result_callback: impl FnMut(T),
) {
    if let Ok(value) = task.result_channel_mut().try_recv() {
        result_callback(value);

        task.is_finished = true;
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
        ui.add(egui::ProgressBar::new(task.current as f32 / task.total as f32).animate(true));
        ui.label(format!("{}/{} downloaded", task.current, task.total));
    } else {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Waiting for the progress data...");
        });
    }

    if let Some(handle) = task.handle.as_ref() {
        if ui.button("Cancel").clicked() {
            handle.abort();
            task.is_finished = true;
        }
    }
}

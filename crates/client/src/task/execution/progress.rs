use std::sync::{Arc, OnceLock};

use eframe::egui::{ProgressBar, Ui};
use nomi_core::downloads::traits::DownloadResult;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::channel::Channel;

pub struct TaskProgress {
    current: u32,
    total: Arc<OnceLock<u32>>,
    channel: Channel<DownloadResult>,
}

impl Default for TaskProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskProgress {
    pub fn new() -> Self {
        Self {
            current: 0,
            total: Arc::new(OnceLock::new()),
            channel: Channel::new(100),
        }
    }

    pub fn ui(&self, ui: &mut Ui) {
        // Value must be initialized
        debug_assert!(self.total.get().is_some());
        if let Some(total) = self.total.get().copied() {
            ui.add(
                ProgressBar::new(self.current as f32 / total as f32)
                    .text(format!("{}/{}", self.current, total)),
            );
        } else {
            ui.spinner();
        }
    }

    pub fn current_mut(&mut self) -> &mut u32 {
        &mut self.current
    }

    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    pub fn sender(&self) -> Sender<DownloadResult> {
        self.channel.clone_tx()
    }

    pub fn receiver_mut(&mut self) -> &mut Receiver<DownloadResult> {
        &mut self.channel
    }

    pub fn share(&self) -> TaskProgressShared {
        TaskProgressShared {
            total: self.total.clone(),
            progress_sender: self.sender(),
        }
    }
}

pub struct TaskProgressShared {
    total: Arc<OnceLock<u32>>,
    progress_sender: Sender<DownloadResult>,
}

impl TaskProgressShared {
    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    pub fn progress_sender(&self) -> Sender<DownloadResult> {
        self.progress_sender.clone()
    }
}

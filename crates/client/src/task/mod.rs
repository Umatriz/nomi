use std::{
    any::Any,
    sync::{Arc, OnceLock},
};

use eframe::egui::{AboveOrBelow, Id, Ui};
use execution::Finished;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::popup::popup;

mod collection;
mod execution;
mod manager;

pub use collection::*;
pub use execution::*;
pub use manager::*;

pub type AnyTask = Task<Box<dyn Any + Send>>;

pub struct Task<R> {
    name: String,
    is_finished: Arc<OnceLock<Finished>>,
    inner: Caller<R>,
}

impl<R: 'static> Task<R> {
    pub fn new(name: impl Into<String>, caller: Caller<R>) -> Self {
        Self {
            name: name.into(),
            is_finished: Arc::new(OnceLock::new()),
            inner: caller,
        }
    }

    fn execute<C, F>(self, channel: Sender<C>, mapper: F) -> TaskData
    where
        C: Send + 'static,
        F: FnOnce(R) -> C + Send + 'static,
    {
        let spawn_future = |fut| {
            let is_finished = self.is_finished.clone();
            tokio::spawn(async move {
                let value = fut.await;
                let value = mapper(value);
                let _ = channel.send(value).await;
                let _ = is_finished.set(Finished);
            })
        };

        let (fut, progress) = match self.inner {
            Caller::Standard(fut) => (fut, None),
            Caller::Progressing(fun) => {
                let task_progress = TaskProgress::new();
                let fut = (fun)(task_progress.share());

                (fut, Some(task_progress))
            }
        };

        let handle = spawn_future(fut);

        TaskData {
            name: self.name,
            handle,
            is_finished: self.is_finished.clone(),
            progress,
        }
    }
}

impl<R: Send + 'static> Task<R> {
    pub fn into_any(self) -> AnyTask {
        Task {
            name: self.name,
            is_finished: self.is_finished,
            inner: self.inner.into_any(),
        }
    }
}

pub struct TaskData {
    name: String,
    handle: JoinHandle<()>,
    is_finished: Arc<OnceLock<Finished>>,
    progress: Option<TaskProgress>,
}

impl TaskData {
    pub fn ui(&self, ui: &mut Ui) {
        ui.label(self.name.as_str());
        match self.progress.as_ref() {
            Some(progress) => progress.ui(ui),
            None => {
                ui.spinner();
            }
        }

        let button = ui.button("Cancel");
        let popup_id = Id::new("confirm_task_cancellation_popup_id");

        popup(ui, popup_id, &button, AboveOrBelow::Below, |ui, state| {
            ui.label("Are you sure you want to cancel the task?");
            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    state.close();
                    self.handle.abort();
                    let _ = self.is_finished.set(Finished);
                };
                if ui.button("No").clicked() {
                    state.close();
                };
            });
        });
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished.get().is_some()
    }

    pub fn progress(&self) -> Option<&TaskProgress> {
        self.progress.as_ref()
    }

    pub fn progress_mut(&mut self) -> Option<&mut TaskProgress> {
        self.progress.as_mut()
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use super::*;

//     #[tokio::test]
//     async fn manager_test() {
//         struct IntCollection;

//         impl<'c> TasksCollection<'c> for IntCollection {
//             type Context = ();

//             type Target = i32;

//             type Executor = LinearTasksExecutor;

//             fn name() -> &'static str {
//                 "Integer collection"
//             }

//             fn handle(_context: Self::Context) -> TaskHandle<'c, Self::Target> {
//                 TaskHandle::new(|int| println!("{}", int))
//             }
//         }

//         async fn task() -> i32 {
//             println!("Started");
//             tokio::time::sleep(Duration::from_secs(1)).await;
//             1
//         }

//         let mut manager = TasksManager::new();
//         manager.add_collection::<IntCollection>(());

//         manager.push_task::<IntCollection>(Task::new("Task", Caller::standard(task())));

//         manager.listen_collection::<IntCollection>();

//         tokio::time::sleep(Duration::from_secs(2)).await;

//         manager.listen_collection::<IntCollection>();

//         task().await;
//     }
// }

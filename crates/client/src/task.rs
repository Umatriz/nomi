use std::{
    any::{type_name, Any, TypeId},
    cell::Cell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

use eframe::{
    egui::{AboveOrBelow, Id, ProgressBar, Ui},
    glow::BOOL,
};
use nomi_core::downloads::traits::{DownloadResult, Downloader};
use pollster::FutureExt;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{channel::Channel, popup::popup};

#[derive(Default)]
pub struct TasksManager<'c> {
    collections: HashMap<TypeId, CollectionData<'c>>,
}

impl TasksManager<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct CollectionData<'c> {
    name: &'static str,
    handle: AnyTaskHandle<'c>,
    tasks: Vec<TaskData>,
}

impl<'c> CollectionData<'c> {
    pub fn from_collection<C: TasksCollection<'c>>(context: C::Context) -> Self {
        Self {
            name: C::name(),
            handle: C::handle(context).into_any(),
            tasks: Vec::new(),
        }
    }
}

pub trait TasksCollection<'c> {
    type Context: 'c;
    type Target: 'static;

    fn name() -> &'static str;
    fn handle(context: Self::Context) -> TaskHandle<'c, Self::Target>;
}

impl<'c> TasksManager<'c> {
    pub fn add_collection<C>(&mut self, context: C::Context)
    where
        C: TasksCollection<'c> + 'static,
    {
        self.collections.insert(
            TypeId::of::<C>(),
            CollectionData::from_collection::<C>(context),
        );
    }

    pub fn push_task<C, Fut>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c> + 'static,
        C::Target: Send + 'static,
    {
        let mapper = |r: C::Target| Box::new(r) as Box<dyn Any + Send>;

        let collection_data = self
            .collections
            .get_mut(&TypeId::of::<C>())
            .unwrap_or_else(|| {
                panic!(
                    "You must add `{}` collection to the `TaskManager` by calling `add_collection`",
                    type_name::<C>()
                )
            });

        let channel = collection_data.handle.channel.clone_tx();

        let task_data = task.execute(channel, mapper);
        collection_data.tasks.push(task_data);
    }
}

type AnyTaskHandle<'c> = TaskHandle<'c, Box<dyn Any + Send>>;

pub struct TaskHandle<'c, T> {
    channel: Channel<T>,
    handle: Box<dyn FnMut(T) + 'c>,
}

impl<'c, T: 'static> TaskHandle<'c, T> {
    pub fn from_handle(handle: impl FnMut(T) + 'c) -> Self {
        Self {
            channel: Channel::new(100),
            handle: Box::new(handle),
        }
    }

    pub fn into_any(mut self) -> AnyTaskHandle<'c> {
        TaskHandle {
            channel: Channel::new(100),
            handle: Box::new(move |boxed_any| {
                let reference = boxed_any.downcast().unwrap();
                (self.handle)(*reference)
            }),
        }
    }

    pub fn listen(&mut self) {
        if let Ok(data) = self.channel.try_recv() {
            (self.handle)(data)
        }
    }
}

pub type AnyTask = Task<Box<dyn Any>>;

pub struct Task<R> {
    name: String,
    is_finished: Arc<OnceLock<Finished>>,
    inner: Caller<R>,
}

impl<R: Send + 'static> Task<R> {
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

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub enum Caller<T> {
    Standard(PinnedFuture<T>),
    Progressing(Box<dyn FnOnce(TaskProgressShared) -> PinnedFuture<T>>),
}

impl<T> Caller<T> {
    pub fn standard<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Standard(Box::pin(fut))
    }

    pub fn progressing<F, Fut>(fun: F) -> Self
    where
        F: FnOnce(TaskProgressShared) -> Fut + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Progressing(Box::new(|progress| Box::pin(fun(progress))))
    }
}

/// Marker to determine if task is finished
struct Finished;

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

    pub fn progress(&self) -> Option<&TaskProgress> {
        self.progress.as_ref()
    }

    pub fn progress_mut(&mut self) -> Option<&mut TaskProgress> {
        self.progress.as_mut()
    }
}

pub struct TaskProgress {
    current: u32,
    total: Arc<OnceLock<u32>>,
    progress_channel: Channel<DownloadResult>,
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
            progress_channel: Channel::new(100),
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

    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    pub fn progress_sender(&self) -> Sender<DownloadResult> {
        self.progress_channel.clone_tx()
    }

    pub fn share(&self) -> TaskProgressShared {
        TaskProgressShared {
            total: self.total.clone(),
            progress_sender: self.progress_sender(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_test() {
        struct AssetsCollection;

        impl<'c> TasksCollection<'c> for AssetsCollection {
            type Context = &'c mut i32;
            type Target = ();

            fn name() -> &'static str {
                "Assets collection"
            }

            fn handle(context: Self::Context) -> TaskHandle<'c, Self::Target> {
                TaskHandle::from_handle(move |()| {
                    *context += 1;
                    println!("Asset received {}", context);
                })
            }
        }

        struct IOCollection;

        impl<'c> TasksCollection<'c> for IOCollection {
            type Context = &'c mut String;
            type Target = ();

            fn name() -> &'static str {
                "IO collection"
            }

            fn handle(context: Self::Context) -> TaskHandle<'c, Self::Target> {
                TaskHandle::from_handle(move |()| {
                    context.push('1');
                    println!("IO {}", context);
                })
            }
        }

        let mut state = (5, "String".to_owned());

        let mut manager = TasksManager::new();
        manager.add_collection::<AssetsCollection>(&mut state.0);
        manager.add_collection::<IOCollection>(&mut state.1);
    }
}

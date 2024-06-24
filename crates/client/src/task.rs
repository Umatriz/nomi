use std::{
    any::{type_name, Any, TypeId},
    cell::Cell,
    collections::{HashMap, VecDeque},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, OnceLock},
};

use eframe::{
    egui::{AboveOrBelow, Frame, Id, ProgressBar, Ui},
    glow::BOOL,
};
use nomi_core::downloads::traits::{DownloadResult, Downloader};
use pollster::FutureExt;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::error;

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

fn task_any_mapper<'c, C>(source: C::Target) -> Box<dyn Any + Send>
where
    C: TasksCollection<'c>,
    C::Target: Send,
{
    Box::new(source)
}

impl<'c> TasksManager<'c> {
    fn get_collection_mut<C>(&mut self) -> &mut CollectionData<'c>
    where
        C: TasksCollection<'c> + 'static,
    {
        self.collections
            .get_mut(&TypeId::of::<C>())
            .unwrap_or_else(move || {
                panic!(
                    "You must add `{}` collection to the `TaskManager` by calling `add_collection`",
                    type_name::<C>()
                )
            })
    }

    pub fn add_collection<C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c> + 'static,
        C::Executor: Default + 'static,
    {
        self.collections.insert(
            TypeId::of::<C>(),
            CollectionData::from_collection::<C>(context),
        );
        self
    }

    pub fn listen_collection<C>(&mut self)
    where
        C: TasksCollection<'c> + 'static,
    {
        self.get_collection_mut::<C>().listen()
    }

    pub fn push_task<C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task::<C>(task);
    }
}

pub struct CollectionData<'c> {
    name: &'static str,
    handle: AnyTaskHandle<'c>,
    tasks: Vec<TaskData>,
    executor: Box<dyn TasksExecutor<'c>>,
}

impl<'c> CollectionData<'c> {
    pub fn ui(&self, ui: &mut Ui) {
        ui.label(self.name);

        for task in &self.tasks {
            ui.group(|ui| {
                task.ui(ui);
            });
        }
    }

    pub fn from_collection<C>(context: C::Context) -> Self
    where
        C: TasksCollection<'c>,
        C::Executor: Default + 'static,
    {
        Self {
            name: C::name(),
            handle: C::handle(context).into_any(),
            tasks: Vec::new(),
            executor: Box::<C::Executor>::default(),
        }
    }

    fn execute(&mut self, task: AnyTask) {
        let channel = self.handle.channel.clone_tx();
        let task_data = task.execute(channel, |t| t);
        self.push_task_data(task_data);
    }

    fn push_task_data(&mut self, task_data: TaskData) {
        self.tasks.push(task_data)
    }

    pub fn push_task<C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c>,
        C::Target: Send,
    {
        self.executor.push(task.into_any());
    }

    pub fn listen(&mut self) {
        self.listen_execution();
        self.listen_results();
        self.listen_progress();
    }

    pub fn listen_results(&mut self) {
        self.handle.listen()
    }

    pub fn listen_progress(&mut self) {
        for task in self.tasks.iter_mut() {
            let Some(progress) = task.progress_mut() else {
                continue;
            };

            if let Ok(result) = progress.receiver_mut().try_recv() {
                progress.current += result.inspect_err(|e| error!("{}", e)).map_or(0, |_| 1);
            }
        }
    }

    pub fn listen_execution(&mut self) {
        use ExecutionPoll as E;
        while let E::Ready(task) = self.executor.poll(&self.tasks) {
            self.execute(task)
        }
    }
}

pub trait TasksCollection<'c> {
    type Context: 'c;
    type Target: Send + 'static;
    type Executor: TasksExecutor<'c>;

    fn name() -> &'static str;
    fn handle(context: Self::Context) -> TaskHandle<'c, Self::Target>;
}

const _: Option<Box<dyn TasksExecutor<'static>>> = None;

pub trait TasksExecutor<'c> {
    fn push(&mut self, task: AnyTask);
    fn poll(&mut self, tasks: &[TaskData]) -> ExecutionPoll;
}

pub enum ExecutionPoll {
    /// There's a task ready to be executed. [`TasksExecutor::execute`] must be called.
    Ready(AnyTask),
    /// There's no tasks or you're waiting for others to finish.
    Pending,
}

#[derive(Default)]
pub struct LinearTasksExecutor {
    inner: VecDeque<AnyTask>,
}

impl<'c> TasksExecutor<'c> for LinearTasksExecutor {
    fn push(&mut self, task: AnyTask) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, tasks: &[TaskData]) -> ExecutionPoll {
        if !self.inner.is_empty() && !tasks.is_empty() {
            return ExecutionPoll::Pending;
        }

        self.inner
            .pop_front()
            .map_or(ExecutionPoll::Pending, ExecutionPoll::Ready)
    }
}

type AnyTaskHandle<'c> = TaskHandle<'c, Box<dyn Any + Send>>;

pub struct TaskHandle<'c, T> {
    channel: Channel<T>,
    handle: Box<dyn FnMut(T) + 'c>,
}

impl<'c, T: 'static> TaskHandle<'c, T> {
    pub fn new(handle: impl FnMut(T) + 'c) -> Self {
        Self {
            channel: Channel::new(100),
            handle: Box::new(handle),
        }
    }

    pub fn into_any(mut self) -> AnyTaskHandle<'c> {
        TaskHandle {
            channel: Channel::new(100),
            handle: Box::new(move |boxed_any| {
                let reference = boxed_any.downcast::<T>().unwrap();
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
        Self::Progressing(Box::new(|progress| Box::pin((fun)(progress))))
    }
}

impl<T: Send + 'static> Caller<T> {
    pub fn into_any(self) -> Caller<Box<dyn Any + Send>> {
        match self {
            Self::Standard(fut) => Caller::standard(Box::pin(async move {
                Box::new(fut.await) as Box<dyn Any + Send>
            })),
            Self::Progressing(fun) => {
                let fun = Box::new(|progress| {
                    let fut = (fun)(progress);
                    Box::pin(async move { Box::new(fut.await) as Box<dyn Any + Send> })
                });

                Caller::progressing(fun)
            }
        }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn manager_test() {
        struct IntCollection;

        impl<'c> TasksCollection<'c> for IntCollection {
            type Context = ();

            type Target = i32;

            type Executor = LinearTasksExecutor;

            fn name() -> &'static str {
                "Integer collection"
            }

            fn handle(_context: Self::Context) -> TaskHandle<'c, Self::Target> {
                TaskHandle::new(|int| println!("{}", int))
            }
        }

        async fn task() -> i32 {
            println!("Started");
            tokio::time::sleep(Duration::from_secs(1)).await;
            1
        }

        let mut manager = TasksManager::new();
        manager.add_collection::<IntCollection>(());

        manager.push_task::<IntCollection>(Task::new("Task", Caller::standard(task())));

        manager.listen_collection::<IntCollection>();

        tokio::time::sleep(Duration::from_secs(2)).await;

        manager.listen_collection::<IntCollection>();

        task().await;
    }
}

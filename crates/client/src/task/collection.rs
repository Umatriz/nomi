use std::any::Any;

use eframe::egui::Ui;
use tracing::error;

use crate::channel::Channel;

use super::{
    execution::{Handle, TasksExecutor},
    AnyHandle, AnyTask, Task, TaskData,
};

pub trait TasksCollection<'c> {
    type Context: 'c;
    type Target: Send + 'static;
    type Executor: TasksExecutor;

    fn name() -> &'static str;
    fn handle(context: Self::Context) -> Handle<'c, Self::Target>;
}

pub struct CollectionData {
    name: &'static str,
    channel: Channel<Box<dyn Any + Send>>,
    tasks: Vec<TaskData>,
    executor: Box<dyn TasksExecutor>,
}

impl CollectionData {
    pub fn ui(&self, ui: &mut Ui) {
        ui.collapsing(self.name, |ui| {
            for task in &self.tasks {
                ui.group(|ui| {
                    task.ui(ui);
                });
            }
        });
    }

    pub(super) fn from_collection<'c, C>() -> Self
    where
        C: TasksCollection<'c>,
        C::Executor: Default + 'static,
    {
        Self {
            name: C::name(),
            channel: Channel::new(100),
            tasks: Vec::new(),
            executor: Box::<C::Executor>::default(),
        }
    }

    fn execute(&mut self, task: AnyTask) {
        let sender = self.channel.clone_tx();
        let task_data = task.execute(sender);
        self.push_task_data(task_data);
    }

    fn push_task_data(&mut self, task_data: TaskData) {
        self.tasks.push(task_data)
    }

    pub fn push_task<'c, C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c>,
        C::Target: Send,
    {
        self.executor.push(task.into_any());
    }

    pub fn handle_all(&mut self, handle: AnyHandle<'_>) {
        self.handle_execution();
        self.handle_progress();
        self.handle_results(handle);
        self.handle_deletion();
    }

    pub fn handle_results(&mut self, mut handle: AnyHandle<'_>) {
        if let Ok(value) = self.channel.try_recv() {
            handle.apply(value)
        }
    }

    pub fn handle_deletion(&mut self) {
        self.tasks.retain(|task| !task.is_finished())
    }

    pub fn handle_progress(&mut self) {
        for task in self.tasks.iter_mut() {
            let Some(progress) = task.progress_mut() else {
                continue;
            };

            if let Ok(result) = progress.receiver_mut().try_recv() {
                *progress.current_mut() += result.inspect_err(|e| error!("{}", e)).map_or(0, |_| 1);
            }
        }
    }

    pub fn handle_execution(&mut self) {
        use crate::task::execution::ExecutionPoll as E;
        while let E::Ready(task) = self.executor.poll(&self.tasks) {
            self.execute(task)
        }
    }
}

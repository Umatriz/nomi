use eframe::egui::Ui;
use tracing::error;

use super::{
    execution::{AnyTaskHandle, TaskHandle, TasksExecutor},
    AnyTask, Task, TaskData,
};

pub trait TasksCollection<'c> {
    type Context: 'c;
    type Target: Send + 'static;
    type Executor: TasksExecutor<'c>;

    fn name() -> &'static str;
    fn handle(context: Self::Context) -> TaskHandle<'c, Self::Target>;
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
        let channel = self.handle.channel().clone_tx();
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

    pub fn listen_all(&mut self) {
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
                *progress.current_mut() += result.inspect_err(|e| error!("{}", e)).map_or(0, |_| 1);
            }
        }
    }

    pub fn listen_execution(&mut self) {
        use crate::task::execution::ExecutionPoll as E;
        while let E::Ready(task) = self.executor.poll(&self.tasks) {
            self.execute(task)
        }
    }
}

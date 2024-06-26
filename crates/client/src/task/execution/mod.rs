use std::{future::Future, pin::Pin};

use super::{AnyTask, TaskData};

mod caller;
pub mod executors;
mod handle;
mod progress;

pub use caller::Caller;
pub use handle::*;
pub use progress::*;

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Marker to determine if task is finished
pub(super) struct Finished;

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

use std::collections::VecDeque;

use crate::task::{AnyTask, TaskData};

use super::{ExecutionPoll, TasksExecutor};

/// Provides linear tasks execution.
///
/// Only one task might be executed at the time. The new task
/// will be executed as soon as the previous one is finished.
///
/// The underlying implementation uses [`VecDeque`].
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

/// Provides parallel tasks execution.
///
/// Several tasks might be executed at the time. The new task
/// will be executed immediately.
///
/// The underlying implementation uses [`VecDeque`].
#[derive(Default)]
pub struct ParallelTasksExecutor {
    inner: VecDeque<AnyTask>,
}

impl<'c> TasksExecutor<'c> for ParallelTasksExecutor {
    fn push(&mut self, task: AnyTask) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, _tasks: &[TaskData]) -> ExecutionPoll {
        self.inner
            .pop_front()
            .map_or(ExecutionPoll::Pending, ExecutionPoll::Ready)
    }
}

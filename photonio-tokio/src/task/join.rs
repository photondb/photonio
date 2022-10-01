use std::future::Future;

use futures::FutureExt;
use tokio::task;

use super::{Task, TaskId};

#[derive(Debug)]
pub struct JoinError(task::JoinError);

#[derive(Debug)]
pub struct JoinHandle<T> {
    task: Task,
    handle: task::JoinHandle<T>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn new(handle: task::JoinHandle<T>) -> Self {
        Self {
            task: Task(TaskId),
            handle,
        }
    }

    pub fn task(&self) -> &Task {
        &self.task
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.handle.poll_unpin(cx).map_err(JoinError)
    }
}

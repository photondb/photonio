use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use super::Task;

/// An error returned from joining a task.
#[derive(Debug)]
pub struct JoinError;

/// A handle to await a task.
pub struct JoinHandle<T> {
    task: Task,
    _mark: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub(super) fn new(task: Task) -> Self {
        Self {
            task,
            _mark: PhantomData,
        }
    }

    /// Returns a reference to the task.
    pub fn task(&self) -> &Task {
        &self.task
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        self.task.detach();
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.task.join(cx.waker())
    }
}

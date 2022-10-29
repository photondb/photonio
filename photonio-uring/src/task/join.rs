use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use super::{Result, Task};

/// A handle to await a task.
///
/// A `JoinHandle` detaches the associated task when it is dropped. The task
/// will continue to run on the runtime until it completes.
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

impl<T> Unpin for JoinHandle<T> {}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.task.join(cx.waker())
    }
}

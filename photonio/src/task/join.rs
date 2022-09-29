use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use super::Task;

/// A handle to await a task.
pub struct JoinHandle<T> {
    task: Task,
    _mark: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn task(&self) -> &Task {
        &self.task
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.task.join(cx.waker())
    }
}

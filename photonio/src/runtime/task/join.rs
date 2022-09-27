use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use super::RawTask;

pub struct JoinHandle<T> {
    task: Arc<RawTask>,
    _mark: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub(super) fn new(task: Arc<RawTask>) -> Self {
        Self {
            task,
            _mark: PhantomData,
        }
    }
}

pub struct JoinError;

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.task.join(cx.waker())
    }
}

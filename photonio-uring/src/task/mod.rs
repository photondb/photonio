//! Primitives for asynchronous tasks.
//!
//! This module is similar to [`std::thread`], but for asynchronous tasks
//! instead of threads.

pub use std::thread::Result;
use std::{
    future::Future,
    mem::ManuallyDrop,
    sync::Arc,
    task::{Poll, Waker},
};

pub use crate::runtime::spawn;

mod raw;
use raw::{Head, Suit};

mod join;
pub use join::JoinHandle;

mod yield_now;
pub use yield_now::yield_now;

/// A unique identifier for a task.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(u64);

/// A handle to an asynchronous task.
pub struct Task(ManuallyDrop<Arc<Head>>);

impl Task {
    pub(crate) fn new<F, S>(id: u64, future: F, schedule: S) -> (Self, JoinHandle<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
        S: Schedule + Send + Sync,
    {
        let suit = Arc::new(Suit::new(id, future, schedule));
        let task = Self::from_suit(suit.clone());
        let handle = JoinHandle::new(Self::from_suit(suit));
        (task, handle)
    }

    fn from_suit<F, S>(suit: Arc<Suit<F, S>>) -> Self
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
        S: Schedule + Send + Sync,
    {
        let head = unsafe { Arc::from_raw(Arc::into_raw(suit) as *const Head) };
        Self(ManuallyDrop::new(head))
    }

    /// Returns the unique identifier of this task.
    pub fn id(&self) -> TaskId {
        TaskId(self.0.id())
    }

    pub(crate) fn poll(&self) {
        unsafe { self.0.poll(&self.0) }
    }

    pub(super) fn join<T>(&self, waker: &Waker) -> Poll<Result<T>> {
        unsafe { self.0.join(&self.0, waker) }
    }

    pub(super) fn detach(&self) {
        unsafe { self.0.detach(&self.0) }
    }
}

unsafe impl Send for Task {}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe { self.0.as_ref().drop(&self.0) }
    }
}

pub(crate) trait Schedule {
    fn schedule(&self, task: Task);
}

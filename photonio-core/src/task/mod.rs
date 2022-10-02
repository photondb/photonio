//! Primitives for asynchronous tasks.
//!
//! This module is similar to [`std::thread`], but for asynchronous tasks instead of threads.

use std::{
    future::Future,
    ptr::NonNull,
    task::{Poll, Waker},
};

pub use crate::runtime::spawn;

mod raw;
use raw::RawTask;

mod join;
pub use join::{JoinError, JoinHandle};

mod yield_now;
pub use yield_now::yield_now;

mod schedule;
pub(crate) use schedule::Schedule;

/// A unique identifier for a task.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(u64);

/// A handle to a task.
pub struct Task(NonNull<RawTask>);

impl Task {
    pub(crate) fn new<F, S>(id: u64, future: F, schedule: S) -> (Self, JoinHandle<F::Output>)
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        let task = Self(RawTask::new(id, future, schedule));
        let handle = JoinHandle::new(task.clone());
        (task, handle)
    }

    /// Returns the unique identifier of this task.
    pub fn id(&self) -> TaskId {
        TaskId(self.raw().id())
    }

    pub(crate) fn poll(self) {
        unsafe { self.raw().poll(self.0) }
    }

    pub(super) fn join<T>(&self, waker: &Waker) -> Poll<Result<T, JoinError>> {
        unsafe { self.raw().join(self.0, waker) }
    }

    pub(super) fn detach(&self) {
        unsafe { self.raw().detach(self.0) }
    }
}

impl Task {
    fn raw(&self) -> &RawTask {
        unsafe { self.0.as_ref() }
    }
}

unsafe impl Send for Task {}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe { self.raw().drop(self.0) }
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        unsafe { Self(self.raw().clone(self.0)) }
    }
}

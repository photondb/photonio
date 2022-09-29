//! Types and operations for asynchronous tasks.
//!
//! This module is similar to [`std::thread`], but for asynchronous tasks instead of threads.

use std::{
    future::Future,
    ptr::NonNull,
    sync::Arc,
    task::{Poll, Waker},
};

mod raw;
use raw::{Head, RawTask};

mod join;
pub use join::JoinHandle;

mod schedule;
pub(crate) use schedule::Schedule;

/// A handle to a task.
pub struct Task(NonNull<Head>);

/// A unique identifier for a task.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(u64);

impl Task {
    pub(crate) fn new<F, S>(id: u64, future: F, schedule: S) -> (Self, JoinHandle<F::Output>)
    where
        F: Future + Send,
        F::Output: Send,
        S: Schedule + Send,
    {
        let raw = Arc::new(RawTask::new(id, future, schedule));
        let task = Self::from_raw(raw.clone());
        let handle = JoinHandle::new(Self::from_raw(raw));
        (task, handle)
    }

    pub fn id(&self) -> TaskId {
        TaskId(self.head().id)
    }

    pub(crate) fn poll(self) {
        unsafe {
            (self.head().vtable.poll)(self.0);
        }
    }

    pub(crate) fn join<T>(&self, waker: &Waker) -> Poll<T> {
        let mut result = Poll::Pending;
        unsafe {
            (self.head().vtable.join)(self.0, waker, &mut result as *mut _ as *mut _);
        }
        result
    }

    pub(crate) fn detach(&self) {
        unsafe {
            (self.head().vtable.detach)(self.0);
        }
    }
}

impl Task {
    fn from_raw<F, S>(raw: Arc<RawTask<F, S>>) -> Self
    where
        F: Future,
        S: Schedule,
    {
        let head = unsafe {
            let ptr = Arc::into_raw(raw);
            NonNull::new_unchecked(ptr as *mut Head)
        };
        Self(head)
    }

    fn head(&self) -> &Head {
        unsafe { self.0.as_ref() }
    }
}

unsafe impl Send for Task {}

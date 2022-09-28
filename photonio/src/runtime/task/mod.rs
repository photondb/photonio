use std::{future::Future, sync::Arc};

mod raw;
use raw::RawTask;

mod join;
pub use join::JoinHandle;

pub use crate::runtime::spawn;

pub struct Task {
    raw: Arc<RawTask>,
}

pub struct TaskId(u64);

impl Task {
    pub(crate) fn new<F>(id: u64, future: F) -> Self
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let raw = RawTask::new(id, future);
        Self { raw: Arc::new(raw) }
    }

    pub fn id(&self) -> TaskId {
        TaskId(self.raw.id())
    }
}

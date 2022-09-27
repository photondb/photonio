use std::{future::Future, sync::Arc};

mod raw;
use raw::RawTask;

mod join;
pub use join::JoinHandle;

pub struct Task {
    raw: Arc<RawTask>,
}

impl Task {
    pub fn new<F>(id: u64, future: F) -> Self
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let raw = RawTask::new(id, future);
        Self { raw: Arc::new(raw) }
    }

    pub fn run(self) {
        todo!()
    }
}

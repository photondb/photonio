use std::{
    future::Future,
    io::Result,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{Builder, Worker};
use crate::task::JoinHandle;

pub(super) struct Scheduler {
    workers: Vec<Worker>,
    next_id: AtomicU64,
}

impl Scheduler {
    pub fn build(builder: &Builder) -> Result<Self> {
        let mut workers = Vec::new();
        let num_threads = builder.num_threads.unwrap_or_else(|| num_cpus::get());
        for id in 0..num_threads {
            let worker = Worker::spawn(id, builder)?;
            workers.push(worker);
        }
        Ok(Self {
            workers,
            next_id: AtomicU64::new(0),
        })
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let task_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let worker_id = (task_id % self.workers.len() as u64) as usize;
        self.workers[worker_id].schedule(task_id, future)
    }
}

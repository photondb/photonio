use std::{
    future::Future,
    io::Result,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    worker::{Shared, Worker},
    Builder,
};
use crate::task::{JoinHandle, Task};

pub(super) struct Executor {
    shared: Shared,
    workers: Vec<Worker>,
    next_id: AtomicU64,
}

impl Executor {
    pub fn new(builder: Builder) -> Result<Self> {
        let shared = Shared::new();
        let mut workers = Vec::new();
        for id in 0..builder.num_threads {
            let thread_name = format!("photonio-worker/{}", id);
            let worker = Worker::spawn(
                thread_name,
                builder.thread_stack_size,
                builder.event_interval,
            )?;
            workers.push(worker);
        }
        Ok(Self {
            shared,
            workers,
            next_id: AtomicU64::new(0),
        })
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let index = (id % self.workers.len() as u64) as usize;
        let (task, handle) = Task::new(id, future, self.shared.clone());
        self.workers[index].schedule(task);
        handle
    }
}

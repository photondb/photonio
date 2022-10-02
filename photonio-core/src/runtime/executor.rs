use std::{
    future::Future,
    io::Result,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{worker::Worker, Builder};
use crate::task::JoinHandle;

pub(super) struct Executor {
    workers: Vec<Worker>,
    next_id: AtomicU64,
}

impl Executor {
    pub fn new(builder: Builder) -> Result<Self> {
        let mut workers = Vec::new();
        for id in 0..builder.num_threads {
            let worker = Worker::new(
                id,
                format!("photonio-worker/{}", id),
                builder.thread_stack_size,
                builder.event_interval,
            )?;
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
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let index = (id % self.workers.len() as u64) as usize;
        self.workers[index].spawn(id, future)
    }
}

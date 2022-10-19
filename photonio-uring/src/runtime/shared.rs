use std::{
    future::Future,
    io::Result,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use log::trace;

use super::{worker::Worker, Builder};
use crate::task::JoinHandle;

#[derive(Clone)]
pub(super) struct Shared(Arc<Inner>);

struct Inner {
    workers: Vec<Worker>,
    next_id: AtomicU64,
}

impl Shared {
    pub(super) fn new(builder: Builder) -> Result<Self> {
        let mut workers = Vec::new();
        for id in 0..builder.num_threads {
            let worker = Worker::new(id)?;
            workers.push(worker);
        }
        let inner = Inner {
            workers,
            next_id: AtomicU64::new(0),
        };
        let shared = Self(Arc::new(inner));
        for worker in &shared.0.workers {
            worker.launch(
                shared.clone(),
                builder.thread_stack_size,
                builder.event_interval,
            )?;
        }
        Ok(shared)
    }

    pub(super) fn schedule<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Dispatch tasks in a round-robin fashion.
        let id = self.0.next_id.fetch_add(1, Ordering::Relaxed);
        let index = (id % self.0.workers.len() as u64) as usize;
        trace!("dispatch task {} to worker {}", id, index);
        self.0.workers[index].schedule(id, future)
    }
}

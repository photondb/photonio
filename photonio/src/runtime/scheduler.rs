use std::{
    future::Future,
    io::Result,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{Builder, JoinHandle, Worker};

pub struct Scheduler {
    workers: Vec<Worker>,
    next_worker: AtomicUsize,
}

impl Scheduler {
    pub fn new_with(builder: &Builder) -> Result<Self> {
        let mut workers = Vec::new();
        let num_threads = builder.num_threads.unwrap_or_else(|| num_cpus::get());
        for id in 0..num_threads {
            let worker = Worker::spawn(id, builder)?;
            workers.push(worker);
        }
        Ok(Self {
            workers,
            next_worker: AtomicUsize::new(0),
        })
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let worker = self.next_worker();
        worker.schedule(future)
    }

    fn next_worker(&self) -> &Worker {
        let next = self.next_worker.fetch_add(1, Ordering::Relaxed);
        &self.workers[next % self.workers.len()]
    }
}

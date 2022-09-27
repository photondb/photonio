use std::{
    future::Future,
    io::Result,
    pin::pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use scoped_tls::scoped_thread_local;

mod task;

mod worker;
use worker::Worker;

pub struct Builder {
    num_threads: usize,
}

impl Builder {
    pub fn new() -> Self {
        Self { num_threads: 0 }
    }

    pub fn num_threads(&mut self, num_threads: usize) -> &mut Self {
        self.num_threads = num_threads;
        self
    }

    pub fn build(&mut self) -> Result<Runtime> {
        let inner = Inner::new();
        Ok(Runtime {
            inner: Arc::new(inner),
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Runtime {
    inner: Arc<Inner>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        Builder::new().build()
    }

    /// Runs a future to completion.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        todo!()
    }

    /// Spawns a future onto the runtime.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        todo!()
    }
}

pub struct Inner {
    workers: Vec<Arc<Worker>>,
    next_worker: AtomicUsize,
}

impl Inner {
    fn new() -> Self {
        Self {
            workers: Vec::new(),
            next_worker: AtomicUsize::new(0),
        }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let worker = self.next_worker();
    }

    fn next_worker(&self) -> &Worker {
        let next = self.next_worker.fetch_add(1, Ordering::Relaxed);
        &self.workers[next % self.workers.len()]
    }
}

scoped_thread_local!(static CURRENT: Arc<Inner>);

pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|inner| inner.spawn(future))
}

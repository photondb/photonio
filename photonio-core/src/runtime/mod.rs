//! The PhotonIO runtime.

use std::{future::Future, io::Result, sync::Arc};

use futures::executor::block_on;
use scoped_tls::scoped_thread_local;

use crate::task::JoinHandle;

mod builder;
pub use builder::Builder;

mod worker_pool;
use worker_pool::WorkerPool;

mod worker_thread;
use worker_thread::WorkerThread;

/// The PhotonIO runtime.
pub struct Runtime(Arc<WorkerPool>);

impl Runtime {
    /// Creates a new runtime with default options.
    pub fn new() -> Result<Self> {
        Builder::default().build()
    }

    /// Runs a future to completion.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.spawn(future);
        CURRENT.set(&self.0, || block_on(handle))
    }

    /// Spawns a future onto the runtime.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

scoped_thread_local!(static CURRENT: Arc<WorkerPool>);

/// Spawns a task onto the current runtime.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|pool| pool.spawn(future))
}

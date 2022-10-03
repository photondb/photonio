//! The PhotonIO runtime.

use std::{future::Future, io::Result, sync::Arc};

use futures::executor::block_on;
use scoped_tls::scoped_thread_local;

use crate::task::JoinHandle;

mod builder;
pub use builder::Builder;

mod worker;
pub(crate) use worker::syscall;

mod executor;
use executor::Executor;

/// The PhotonIO runtime.
pub struct Runtime(Arc<Executor>);

impl Runtime {
    /// Creates a new runtime with default options.
    pub fn new() -> Result<Self> {
        Builder::new().build()
    }

    /// Runs a future to completion.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        CURRENT
            .set(&self.0, || block_on(self.spawn(future)))
            .unwrap()
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

scoped_thread_local!(static CURRENT: Arc<Executor>);

/// Spawns a task onto the current runtime.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|exec| exec.spawn(future))
}

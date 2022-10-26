//! The PhotonIO runtime.

use std::{future::Future, io::Result};

use futures::executor::block_on;

use crate::task::JoinHandle;

mod builder;
pub use builder::Builder;

mod shared;
use shared::Shared;

mod driver;

mod worker;
pub use worker::{alloc_uring_buf, spawn};

pub(crate) mod syscall;

/// The PhotonIO runtime.
pub struct Runtime(Shared);

impl Runtime {
    /// Creates a runtime with default options.
    pub fn new() -> Result<Self> {
        Builder::new().build()
    }

    /// Runs a future to completion.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // If the task panics, propagates the panic to the caller.
        block_on(self.spawn(future)).unwrap()
    }

    /// Spawns a future onto this runtime.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.schedule(future)
    }
}

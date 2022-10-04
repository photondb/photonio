use std::{future::Future, io::Result};

use tokio::runtime;

use crate::task::JoinHandle;

mod builder;
pub use builder::Builder;

pub struct Runtime(runtime::Runtime);

impl Runtime {
    pub fn new() -> Result<Self> {
        runtime::Runtime::new().map(Self)
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.block_on(future)
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        JoinHandle::new(self.0.spawn(future))
    }
}

impl From<runtime::Runtime> for Runtime {
    fn from(runtime: runtime::Runtime) -> Self {
        Self(runtime)
    }
}

use std::{future::Future, io::Result, sync::Arc};

use scoped_tls::scoped_thread_local;

use crate::task::JoinHandle;

mod builder;
pub use builder::Builder;

mod thread;

mod worker;
use worker::Worker;

mod scheduler;
use scheduler::Scheduler;

pub struct Runtime {
    sched: Arc<Scheduler>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        Builder::default().build()
    }

    /// Runs a future to completion.
    pub fn run<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.spawn(future);
        CURRENT.set(&self.sched, || thread::block_on(handle))
    }

    /// Spawns a future onto the runtime.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.sched.spawn(future)
    }
}

impl Runtime {
    fn with_sched(sched: Scheduler) -> Self {
        Self {
            sched: Arc::new(sched),
        }
    }
}

scoped_thread_local!(static CURRENT: Arc<Scheduler>);

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|sched| sched.spawn(future))
}

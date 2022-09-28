use std::{future::Future, io::Result, sync::Arc};

use scoped_tls::scoped_thread_local;

mod task;
pub use task::JoinHandle;

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

    pub(crate) fn new_with(builder: &Builder) -> Result<Self> {
        let sched = Scheduler::new_with(builder)?;
        Ok(Self {
            sched: Arc::new(sched),
        })
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

scoped_thread_local!(static CURRENT: Arc<Scheduler>);

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    CURRENT.with(|sched| sched.spawn(future))
}

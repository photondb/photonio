use std::{future::Future, io::Result, thread};

use crate::task::JoinHandle;

mod context;
mod driver;
mod op;

pub(crate) mod syscall;

pub(super) struct Worker {
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(
        id: usize,
        thread_name: String,
        thread_stack_size: usize,
        event_interval: usize,
    ) -> Result<Self> {
        let thread = thread::Builder::new()
            .name(thread_name)
            .stack_size(thread_stack_size)
            .spawn(move || context::run(id, event_interval).unwrap())?;
        Ok(Self { thread })
    }

    pub fn spawn<F>(&self, id: u64, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        todo!()
    }
}

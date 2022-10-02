use std::{io::Result, sync::Arc};

use super::{Runtime, WorkerPool};

/// Builds a [`Runtime`] with custom options.
#[derive(Default)]
pub struct Builder {
    pub(super) num_threads: Option<usize>,
    pub(super) thread_stack_size: Option<usize>,
    pub(super) event_interval: Option<usize>,
}

impl Builder {
    /// Creates a builder with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of threads.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = Some(num_threads);
        self
    }

    /// Sets the stack size for each thread.
    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Self {
        self.thread_stack_size = Some(thread_stack_size);
        self
    }

    /// Sets the number of tasks to poll per tick.
    pub fn event_interval(mut self, event_interval: usize) -> Self {
        self.event_interval = Some(event_interval);
        self
    }

    /// Creates a runtime with the options.
    pub fn build(self) -> Result<Runtime> {
        let pool = WorkerPool::build(self)?;
        Ok(Runtime(Arc::new(pool)))
    }
}

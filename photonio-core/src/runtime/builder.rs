use std::{io::Result, sync::Arc};

use super::{Executor, Runtime};

/// Builds a [`Runtime`] with custom options.
pub struct Builder {
    pub(super) num_threads: usize,
    pub(super) thread_stack_size: usize,
    pub(super) event_interval: usize,
}

impl Builder {
    /// Creates a builder with default options.
    pub fn new() -> Self {
        Self {
            num_threads: num_cpus::get(),
            thread_stack_size: 1 << 20,
            event_interval: 61,
        }
    }

    /// Sets the number of threads.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    /// Sets the stack size for each thread.
    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Self {
        self.thread_stack_size = thread_stack_size;
        self
    }

    /// Sets the number of tasks to poll per event loop.
    pub fn event_interval(mut self, event_interval: usize) -> Self {
        self.event_interval = event_interval;
        self
    }

    /// Creates a runtime with the options.
    pub fn build(self) -> Result<Runtime> {
        let pool = Executor::new(self)?;
        Ok(Runtime(Arc::new(pool)))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

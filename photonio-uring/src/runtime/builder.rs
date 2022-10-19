use std::io::Result;

use super::{Runtime, Shared};

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
            thread_stack_size: 2 << 20,
            event_interval: 61,
        }
    }

    /// Sets the number of worker threads to execute tasks.
    ///
    /// The default value is set to the number of CPU cores.
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    /// Sets the stack size for each worker thread.
    ///
    /// The default value is 2 MiB.
    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Self {
        self.thread_stack_size = thread_stack_size;
        self
    }

    /// Sets the number of tasks to poll per event cycle.
    ///
    /// The default value is 61.
    pub fn event_interval(mut self, event_interval: usize) -> Self {
        self.event_interval = event_interval;
        self
    }

    /// Creates a runtime with the specified options.
    pub fn build(self) -> Result<Runtime> {
        let shared = Shared::new(self)?;
        Ok(Runtime(shared))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

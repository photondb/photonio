use std::io::Result;

use tokio::runtime;

use super::Runtime;

pub struct Builder(runtime::Builder);

impl Builder {
    pub fn new() -> Self {
        let mut b = tokio::runtime::Builder::new_multi_thread();
        b.enable_all();
        Self(b)
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.0.worker_threads(num_threads);
        self
    }

    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Self {
        self.0.thread_stack_size(thread_stack_size);
        self
    }

    pub fn event_interval(mut self, event_interval: usize) -> Self {
        self.0.event_interval(event_interval as _);
        self
    }

    pub fn build(mut self) -> Result<Runtime> {
        self.0.build().map(Runtime)
    }
}

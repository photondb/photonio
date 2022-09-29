use std::io::Result;

use super::{Runtime, Scheduler};

#[derive(Default)]
pub struct Builder {
    pub(crate) num_threads: Option<usize>,
    pub(crate) thread_stack_size: Option<usize>,
}

impl Builder {
    pub fn num_threads(&mut self, num_threads: usize) -> &mut Self {
        self.num_threads = Some(num_threads);
        self
    }

    pub fn thread_stack_size(&mut self, thread_stack_size: usize) -> &mut Self {
        self.thread_stack_size = Some(thread_stack_size);
        self
    }

    pub fn build(&self) -> Result<Runtime> {
        let sched = Scheduler::build(self)?;
        Ok(Runtime::with_sched(sched))
    }
}

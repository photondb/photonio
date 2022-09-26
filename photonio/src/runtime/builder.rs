use std::io::Result;

use super::Runtime;

pub struct Builder {
    num_threads: usize,
}

impl Builder {
    pub fn new() -> Self {
        Self { num_threads: 0 }
    }

    pub fn num_threads(&mut self, num_threads: usize) -> &mut Self {
        self.num_threads = num_threads;
        self
    }

    pub fn build(&mut self) -> Result<Runtime> {
        Runtime::new()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

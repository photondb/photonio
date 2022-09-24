pub struct Runtime {}

impl Runtime {
    /// Runs a future to completion.
    pub fn run<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
    }

    /// Spawns a future onto the runtime.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
    }
}

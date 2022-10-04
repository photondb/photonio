use std::future::Future;

use tokio::task;

mod join;
pub use join::{JoinError, JoinHandle};

#[derive(Debug)]
pub struct Task(TaskId);

impl Task {
    pub fn id(&self) -> TaskId {
        // TODO: Implement this with the `tokio_unstable` feature.
        todo!()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId;

pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    JoinHandle::new(task::spawn(future))
}

pub async fn yield_now() {
    task::yield_now().await
}

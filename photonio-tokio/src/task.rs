use tokio::task;

pub struct JoinHandle<T>(task::JoinHandle<T>);

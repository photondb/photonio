use std::{
    future::Future,
    io::Result,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread,
};

use super::{Builder, JoinHandle};
use crate::{
    io::Driver,
    task::{Schedule, Task},
};

pub(super) struct Worker {
    tx: Sender<Task>,
}

impl Worker {
    pub fn spawn(id: usize, builder: &Builder) -> Result<Self> {
        let (tx, rx) = channel();
        let name = format!("photonio-worker/{}", id);
        let mut thread = thread::Builder::new().name(name);
        if let Some(size) = builder.thread_stack_size {
            thread = thread.stack_size(size);
        }
        thread.spawn(move || {
            run(rx).unwrap();
        })?;
        Ok(Self { tx })
    }

    pub fn schedule<F>(&self, id: u64, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (task, handle) = Task::new(id, future, self.tx.clone());
        self.tx.send(task).unwrap();
        handle
    }
}

impl Schedule for Sender<Task> {
    fn schedule(&self, task: Task) {
        self.send(task).unwrap()
    }
}

fn run(rx: Receiver<Task>) -> Result<()> {
    let driver = Driver::new()?;
    loop {
        loop {
            let task = match rx.try_recv() {
                Ok(task) => task,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            };
            task.poll();
        }
        driver.park()?;
    }
}

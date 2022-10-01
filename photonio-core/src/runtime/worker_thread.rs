use std::{
    future::Future,
    io::Result,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread,
};

use crate::{
    io::Driver,
    task::{JoinHandle, Schedule, Task},
};

pub(super) struct WorkerThread {
    tx: Sender<Task>,
}

impl WorkerThread {
    pub fn spawn(thread: thread::Builder) -> Result<Self> {
        let (tx, rx) = channel();
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
    driver.with(|| loop {
        let mut num_tasks = 0;
        loop {
            let task = match rx.try_recv() {
                Ok(task) => task,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            };
            task.poll();
            num_tasks += 1;
        }
        if num_tasks == 0 {
            driver.park()?;
        } else {
            driver.drive()?;
        }
    })
}

use std::{io::Result, sync::mpsc, thread};

use crate::task::Task;

mod driver;
mod op;

mod context;
pub(super) use context::Shared;

pub(crate) mod syscall;

pub(super) struct Worker {
    tx: Sender,
}

impl Worker {
    pub fn spawn(
        id: usize,
        shared: Shared,
        thread_name: String,
        thread_stack_size: usize,
        event_interval: usize,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name(thread_name)
            .stack_size(thread_stack_size)
            .spawn(move || context::run(id, rx, shared, event_interval).unwrap())?;
        Ok(Self { tx })
    }

    pub fn schedule(&self, task: Task) {
        self.tx.send(Message::Schedule(task)).unwrap();
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.tx.send(Message::Shutdown).unwrap();
    }
}

enum Message {
    Shutdown,
    Schedule(Task),
}

type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

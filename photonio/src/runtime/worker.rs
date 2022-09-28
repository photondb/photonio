use std::{
    future::Future,
    io::Result,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use super::{Builder, JoinHandle};
use crate::io::Driver;

pub struct Worker {
    tx: Sender<()>,
    handle: thread::JoinHandle<()>,
}

impl Worker {
    pub fn spawn(id: usize, builder: &Builder) -> Result<Self> {
        let (tx, rx) = channel();
        let name = format!("photonio-worker/{}", id);
        let mut thread = thread::Builder::new().name(name);
        if let Some(size) = builder.thread_stack_size {
            thread = thread.stack_size(size);
        }
        let handle = thread.spawn(move || {
            run(rx).unwrap();
        })?;
        Ok(Self { tx, handle })
    }

    pub fn schedule<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        todo!()
    }
}

fn run(rx: Receiver<()>) -> Result<()> {
    let driver = Driver::new()?;
    todo!()
}

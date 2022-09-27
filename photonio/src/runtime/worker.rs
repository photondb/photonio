use std::{
    future::Future,
    io::Result,
    pin::pin,
    sync::mpsc::{channel, Receiver, Sender},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use super::task::Task;
use crate::io::Driver;

pub struct Worker {
    driver: Driver,
    tx: Sender<Task>,
    rx: Receiver<Task>,
}

impl Worker {
    pub fn new() -> Result<Self> {
        let driver = Driver::new()?;
        let (tx, rx) = channel();
        Ok(Self { driver, tx, rx })
    }

    /// Runs a future to completion.
    pub fn run(&self) {
        let waker = unsafe { Waker::from_raw(DUMMY_RAW_WAKER) };
        let mut cx = Context::from_waker(&waker);

        self.driver.with(|| {})
    }

    /// Spawns a future onto the runtime.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        todo!()
    }
}

const DUMMY_RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &DUMMY_RAW_VTABLE);
const DUMMY_RAW_VTABLE: RawWakerVTable =
    RawWakerVTable::new(|_| DUMMY_RAW_WAKER, |_| (), |_| (), |_| ());

use std::{
    future::Future,
    io::Result,
    pin::pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::io::Driver;

pub struct Runtime {
    driver: Driver,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let driver = Driver::new()?;
        Ok(Self { driver })
    }

    /// Runs a future to completion.
    pub fn run<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        let waker = unsafe { Waker::from_raw(DUMMY_RAW_WAKER) };
        let mut cx = Context::from_waker(&waker);

        self.driver.with(|| {
            let mut future = pin!(future);
            loop {
                if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                    return output;
                }
                self.driver.park().unwrap();
            }
        })
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

use std::{
    io::{Error, ErrorKind, Result},
    task::{Poll, Waker},
};

use io_uring::IoUring;
use slab::Slab;

pub struct Driver {
    io: IoUring,
    ops: OpTable,
}

impl Driver {
    pub fn new() -> Result<Self> {
        let io = IoUring::new(1024)?;
        Ok(Self {
            io,
            ops: OpTable::default(),
        })
    }

    pub fn tick(&mut self) {
        let mut cq = self.io.completion();
        cq.sync();

        for cqe in cq {
            let index = cqe.user_data() as _;
            let result = result_from_return_value(cqe.result());
            self.ops.complete(index, result);
        }
    }

    fn submit(&mut self) -> Result<()> {
        loop {
            match self.io.submit() {
                Ok(_) => {
                    self.io.submission().sync();
                    return Ok(());
                }
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => {}
                    ErrorKind::ResourceBusy => {
                        self.tick();
                    }
                    _ => return Err(e),
                },
            }
        }
    }
}

#[derive(Default)]
enum OpState {
    #[default]
    Queued,
    Polled(Waker),
    Completed(Result<u32>),
}

#[derive(Default)]
struct OpTable(Slab<OpState>);

impl OpTable {
    fn poll(&mut self, index: usize, waker: &Waker) -> Poll<Result<u32>> {
        let state = &mut self.0[index];
        match std::mem::take(state) {
            OpState::Queued => {
                *state = OpState::Polled(waker.clone());
                Poll::Pending
            }
            OpState::Polled(w) => {
                if !w.will_wake(waker) {
                    *state = OpState::Polled(waker.clone());
                }
                Poll::Pending
            }
            OpState::Completed(result) => {
                self.0.remove(index);
                Poll::Ready(result)
            }
        }
    }

    fn complete(&mut self, index: usize, result: Result<u32>) {
        let state = &mut self.0[index];
        match std::mem::take(state) {
            OpState::Queued => {
                *state = OpState::Completed(result);
            }
            OpState::Polled(w) => {
                *state = OpState::Completed(result);
                w.wake();
            }
            OpState::Completed(..) => unreachable!(),
        }
    }
}

fn result_from_return_value(value: i32) -> Result<u32> {
    if value >= 0 {
        Ok(value as u32)
    } else {
        Err(Error::from_raw_os_error(value))
    }
}

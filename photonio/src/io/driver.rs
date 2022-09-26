use std::{
    cell::RefCell,
    future::Future,
    io::{Error, ErrorKind, Result},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use io_uring::{squeue, IoUring};
use scoped_tls::scoped_thread_local;
use slab::Slab;

pub struct Driver {
    inner: RefCell<Inner>,
}

impl Driver {
    pub fn new() -> Result<Self> {
        let inner = Inner::new()?;
        Ok(Self {
            inner: RefCell::new(inner),
        })
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CURRENT.set(&self.inner, f)
    }

    pub fn park(&self) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        inner.submit_and_wait()?;
        inner.tick();
        Ok(())
    }

    pub fn drive(&self) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        inner.submit()?;
        inner.tick();
        Ok(())
    }
}

struct Inner {
    io: IoUring,
    ops: OpTable,
}

impl Inner {
    fn new() -> Result<Self> {
        let io = IoUring::new(1024)?;
        Ok(Self {
            io,
            ops: OpTable::default(),
        })
    }

    fn push(&mut self, mut sqe: squeue::Entry) -> Result<Op> {
        let op = self.ops.insert();
        sqe = sqe.user_data(op.index as _);
        if self.io.submission().is_full() {
            self.submit()?;
        }
        unsafe {
            self.io.submission().push(&sqe).unwrap();
        }
        Ok(op)
    }

    fn tick(&mut self) {
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

    fn submit_and_wait(&mut self) -> Result<usize> {
        loop {
            match self.io.submit_and_wait(1) {
                Ok(n) => {
                    self.io.submission().sync();
                    return Ok(n);
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }
}

pub struct Op {
    table: OpTable,
    index: usize,
}

impl Future for Op {
    type Output = Result<u32>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let index = self.index;
        self.table.poll(index, cx.waker())
    }
}

#[derive(Default)]
enum OpState {
    #[default]
    Inited,
    Polled(Waker),
    Completed(Result<u32>),
}

#[derive(Clone, Default)]
struct OpTable(Rc<RefCell<Slab<OpState>>>);

impl OpTable {
    fn insert(&mut self) -> Op {
        let mut slab = self.0.borrow_mut();
        let index = slab.insert(OpState::Inited);
        Op {
            index,
            table: self.clone(),
        }
    }

    fn poll(&mut self, index: usize, waker: &Waker) -> Poll<Result<u32>> {
        let mut slab = self.0.borrow_mut();
        let state = slab.get_mut(index).unwrap();
        match std::mem::take(state) {
            OpState::Inited => {
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
                slab.remove(index);
                Poll::Ready(result)
            }
        }
    }

    fn complete(&mut self, index: usize, result: Result<u32>) {
        let mut slab = self.0.borrow_mut();
        let state = slab.get_mut(index).unwrap();
        match std::mem::take(state) {
            OpState::Inited => {
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
        Err(Error::from_raw_os_error(-value))
    }
}

scoped_thread_local!(static CURRENT: RefCell<Inner>);

pub fn submit(sqe: squeue::Entry) -> Result<Op> {
    CURRENT.with(|driver| driver.borrow_mut().push(sqe))
}

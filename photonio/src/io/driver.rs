use std::{
    cell::RefCell,
    future::Future,
    io::{Error, ErrorKind, Result},
    mem,
    os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use io_uring::{opcode, squeue, types, IoUring};
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
    waker: EventFd,
}

impl Inner {
    fn new() -> Result<Self> {
        let io = IoUring::new(1024)?;
        let waker = EventFd::new()?;
        Ok(Self {
            io,
            ops: OpTable::default(),
            waker,
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
            let token = cqe.user_data() as _;
            let result = syscall_result(cqe.result());
            self.ops.complete(token, result);
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
    Init,
    Polled(Waker),
    Canceled,
    Completed(Result<u32>),
}

#[derive(Clone, Default)]
struct OpTable(Rc<RefCell<Slab<OpState>>>);

impl OpTable {
    fn insert(&mut self) -> Op {
        let mut table = self.0.borrow_mut();
        let index = table.insert(OpState::Init);
        Op {
            index,
            table: self.clone(),
        }
    }

    fn poll(&mut self, index: usize, waker: &Waker) -> Poll<Result<u32>> {
        let mut table = self.0.borrow_mut();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init => {
                *state = OpState::Polled(waker.clone());
                Poll::Pending
            }
            OpState::Polled(w) => {
                if !w.will_wake(waker) {
                    *state = OpState::Polled(waker.clone());
                }
                Poll::Pending
            }
            OpState::Canceled => unreachable!(),
            OpState::Completed(result) => {
                table.remove(index);
                Poll::Ready(result)
            }
        }
    }

    fn cancel(&mut self, index: usize) {
        let mut table = self.0.borrow_mut();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init | OpState::Polled(_) => {
                *state = OpState::Canceled;
            }
            OpState::Canceled => unreachable!(),
            OpState::Completed(_) => {
                table.remove(index);
            }
        }
    }

    fn complete(&mut self, index: usize, result: Result<u32>) {
        let mut table = self.0.borrow_mut();
        let state = table.get_mut(index).unwrap();
        match mem::take(state) {
            OpState::Init => {
                *state = OpState::Completed(result);
            }
            OpState::Polled(w) => {
                *state = OpState::Completed(result);
                w.wake();
            }
            OpState::Canceled => {}
            OpState::Completed(..) => unreachable!(),
        }
    }
}

struct EventFd {
    fd: OwnedFd,
}

impl EventFd {
    const TOKEN: u64 = 1 << 63;

    fn new() -> Result<Self> {
        let fd = unsafe {
            let fd = syscall_result(libc::eventfd(0, libc::EFD_CLOEXEC))?;
            OwnedFd::from_raw_fd(fd as RawFd)
        };
        Ok(Self { fd })
    }

    fn wake(&self) -> Result<()> {
        let buf = 1u64.to_ne_bytes();
        let res = unsafe { libc::write(self.fd.as_raw_fd(), buf.as_ptr() as _, buf.len() as _) };
        syscall_result(res as _).map(|_| ())
    }

    fn wait_sqe(&self, buf: &mut [u8; 8]) -> squeue::Entry {
        opcode::Read::new(
            types::Fd(self.fd.as_raw_fd()),
            buf.as_mut_ptr(),
            buf.len() as _,
        )
        .build()
        .user_data(Self::TOKEN)
    }
}

fn syscall_result(res: i32) -> Result<u32> {
    if res >= 0 {
        Ok(res as u32)
    } else {
        Err(Error::from_raw_os_error(-res))
    }
}

scoped_thread_local!(static CURRENT: RefCell<Inner>);

pub fn submit(sqe: squeue::Entry) -> Result<Op> {
    CURRENT.with(|inner| inner.borrow_mut().push(sqe))
}

use std::{
    io::{Error, ErrorKind, Result},
    os::unix::io::{AsRawFd, FromRawFd, OwnedFd},
    sync::Arc,
};

use io_uring::{opcode, squeue, types, IoUring};

mod op;
pub(super) use op::Op;

mod optable;
use optable::OpTable;

pub(super) struct Driver {
    io: IoUring,
    table: OpTable,
    eventfd: Arc<OwnedFd>,
    eventbuf: [u8; 8],
}

impl Driver {
    pub(super) fn new(unpark: Unpark) -> Result<Self> {
        let io = IoUring::new(4096)?;
        Ok(Self {
            io,
            table: OpTable::new(),
            eventfd: unpark.0,
            eventbuf: [0; 8],
        })
    }

    pub(super) unsafe fn add(&mut self, sqe: squeue::Entry) -> Result<Op> {
        let index = self.table.add();
        assert!(index as u64 != Self::UNPARK_TOKEN);
        self.push(sqe.user_data(index as u64))?;
        Ok(Op::new(self.table.clone(), index))
    }

    pub(super) fn tick(&mut self) -> Result<()> {
        self.submit()?;
        self.pull();
        Ok(())
    }

    pub(super) fn park(&mut self) -> Result<()> {
        // Registers the eventfd to unpark this driver.
        let fd = types::Fd(self.eventfd.as_raw_fd());
        let buf = &mut self.eventbuf;
        let sqe = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as _)
            .build()
            .user_data(Self::UNPARK_TOKEN);
        unsafe {
            self.push(sqe)?;
        }
        self.submit_and_wait(1)?;
        self.pull();
        Ok(())
    }
}

impl Driver {
    const UNPARK_TOKEN: u64 = u64::MAX;

    unsafe fn push(&mut self, sqe: squeue::Entry) -> Result<()> {
        while {
            let mut sq = self.io.submission();
            sq.push(&sqe)
        }
        .is_err()
        {
            self.submit()?;
        }
        Ok(())
    }

    fn pull(&mut self) {
        let mut cq = self.io.completion();
        cq.sync();
        for cqe in cq {
            let token = cqe.user_data();
            if token != Self::UNPARK_TOKEN {
                let result = syscall_result(cqe.result());
                self.table.complete(token as _, result);
            }
        }
    }

    fn submit(&mut self) -> Result<usize> {
        self.submit_and_wait(0)
    }

    fn submit_and_wait(&mut self, want: usize) -> Result<usize> {
        loop {
            match self.io.submit_and_wait(want) {
                Ok(n) => {
                    self.io.submission().sync();
                    return Ok(n);
                }
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => {}
                    ErrorKind::ResourceBusy => {
                        self.pull();
                    }
                    _ => return Err(e),
                },
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct Unpark(Arc<OwnedFd>);

impl Unpark {
    pub(super) fn new() -> Result<Self> {
        let fd = unsafe {
            let fd = syscall_result(libc::eventfd(0, libc::EFD_CLOEXEC))?;
            OwnedFd::from_raw_fd(fd as _)
        };
        Ok(Self(Arc::new(fd)))
    }

    pub(super) fn unpark(&self) -> Result<()> {
        let buf = 1u64.to_ne_bytes();
        let ret = unsafe { libc::write(self.0.as_raw_fd(), buf.as_ptr() as _, buf.len() as _) };
        if ret >= 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }
}

fn syscall_result(res: i32) -> Result<u32> {
    if res >= 0 {
        Ok(res as u32)
    } else {
        Err(Error::from_raw_os_error(-res))
    }
}

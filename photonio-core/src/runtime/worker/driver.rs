use std::{
    io::{Error, ErrorKind, Result},
    os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd},
};

use io_uring::{opcode, squeue, types, IoUring};

use super::op::{OpHandle, OpTable};

pub(crate) struct Driver {
    io: IoUring,
    table: OpTable,
}

impl Driver {
    pub fn new() -> Result<Self> {
        let io = IoUring::new(1024)?;
        Ok(Self {
            io,
            table: OpTable::new(),
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        self.submit()?;
        self.pull();
        Ok(())
    }

    pub fn park(&mut self) -> Result<bool> {
        self.submit_and_wait()?;
        self.pull();
        Ok(true)
    }

    pub fn push(&mut self, op: squeue::Entry) -> Result<OpHandle> {
        let handle = self.table.insert();
        let sqe = op.user_data(handle.index() as _);
        if self.io.submission().is_full() {
            self.submit()?;
        }
        unsafe {
            self.io.submission().push(&sqe).unwrap();
        }
        Ok(handle)
    }

    fn pull(&mut self) {
        let mut cq = self.io.completion();
        cq.sync();

        for cqe in cq {
            let token = cqe.user_data();
            let result = syscall_result(cqe.result());
            if token == Unpark::TOKEN {
                result.unwrap();
            } else {
                self.table.complete(token as _, result);
            }
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
                        self.pull();
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

pub struct Unpark {
    fd: OwnedFd,
    buf: [u8; 8],
    is_registered: bool,
}

impl Unpark {
    const TOKEN: u64 = u64::MAX;

    fn new() -> Result<Self> {
        let fd = unsafe {
            let fd = syscall_result(libc::eventfd(0, libc::EFD_CLOEXEC))?;
            OwnedFd::from_raw_fd(fd as RawFd)
        };
        Ok(Self {
            fd,
            buf: [0; 8],
            is_registered: false,
        })
    }

    fn unpark(&self) -> Result<()> {
        let buf = 1u64.to_ne_bytes();
        let res = unsafe { libc::write(self.fd.as_raw_fd(), buf.as_ptr() as _, buf.len() as _) };
        syscall_result(res as _).map(|_| ())
    }

    fn register(&mut self) -> Option<squeue::Entry> {
        if self.is_registered {
            return None;
        }
        self.is_registered = true;
        let sqe = opcode::Read::new(
            types::Fd(self.fd.as_raw_fd()),
            self.buf.as_mut_ptr(),
            self.buf.len() as _,
        )
        .build()
        .user_data(Self::TOKEN);
        Some(sqe)
    }

    fn complete(&mut self) {
        self.is_registered = false;
    }
}

fn syscall_result(res: i32) -> Result<u32> {
    if res >= 0 {
        Ok(res as u32)
    } else {
        Err(Error::from_raw_os_error(-res))
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

use super::syscall_result;

pub struct Unpark(Arc<Inner>);

struct Inner {
    fd: OwnedFd,
    is_parked: AtomicBool,
}

impl Inner {
    fn new() -> Result<Self> {
        let fd = unsafe {
            let fd = syscall_result(libc::eventfd(0, libc::EFD_CLOEXEC))?;
            OwnedFd::from_raw_fd(fd as RawFd)
        };
        Ok(Self {
            fd,
            is_parked: false,
        })
    }

    fn unpark(&self) -> Result<()> {
        if self
            .is_parked
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            let buf = 1u64.to_ne_bytes();
            let res =
                unsafe { libc::write(self.fd.as_raw_fd(), buf.as_ptr() as _, buf.len() as _) };
            syscall_result(res as _).map(|_| ())
        }
    }

    fn opcode(&self, buf: &[u8; 8]) -> squeue::Entry {}
}

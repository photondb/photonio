use std::{
    ffi::CString,
    future::Future,
    io::{Error, ErrorKind, Result},
    os::unix::{ffi::OsStrExt, io::RawFd},
    path::Path,
};

use io_uring::{opcode, types};

use crate::io::submit;

pub fn open(
    path: &Path,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> impl Future<Output = Result<RawFd>> {
    let str = CString::new(path.as_os_str().as_bytes());
    async move {
        let str = str.map_err(|_| Error::from(ErrorKind::InvalidFilename))?;
        let sqe = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), str.as_c_str().as_ptr())
            .flags(flags)
            .mode(mode)
            .build();
        submit(sqe)?.await.map(|v| v as _)
    }
}

pub fn close(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Close::new(types::Fd(fd)).build();
        submit(sqe)?.await.map(|_| ())
    }
}

pub fn read<'a>(fd: RawFd, buf: &'a mut [u8]) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as _).build();
        submit(sqe)?.await.map(|v| v as _)
    }
}

pub fn write<'a>(fd: RawFd, buf: &'a [u8]) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as _).build();
        submit(sqe)?.await.map(|v| v as _)
    }
}

pub fn statx(fd: RawFd) -> impl Future<Output = Result<libc::statx>> {
    async move {
        let mut sbuf = unsafe { std::mem::zeroed() };
        let sqe = opcode::Statx::new(
            types::Fd(fd),
            std::ptr::null(),
            &mut sbuf as *mut _ as *mut _,
        )
        .build();
        submit(sqe)?.await.map(|_| sbuf)
    }
}

pub fn sync_all(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Fsync::new(types::Fd(fd)).build();
        submit(sqe)?.await.map(|_| ())
    }
}

pub fn sync_data(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Fsync::new(types::Fd(fd))
            .flags(types::FsyncFlags::DATASYNC)
            .build();
        submit(sqe)?.await.map(|_| ())
    }
}

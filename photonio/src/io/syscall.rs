//! Asynchronous system calls.

use std::{
    ffi::CString,
    future::Future,
    io::{Error, ErrorKind, Result},
    mem,
    os::unix::{ffi::OsStrExt, io::RawFd},
    path::Path,
};

use io_uring::{opcode, types};
use socket2::SockAddr;

use super::submit;

/// See `man read.2` for more details.
pub fn read<'a>(fd: RawFd, buf: &'a mut [u8]) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as _).build();
        submit(sqe)?.await.map(|n| n as _)
    }
}

/// See `man pread.2` for more details.
pub fn pread<'a>(
    fd: RawFd,
    buf: &'a mut [u8],
    pos: u64,
) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as _)
            .offset(pos as _)
            .build();
        submit(sqe)?.await.map(|n| n as _)
    }
}

/// See `man write.2` for more details.
pub fn write<'a>(fd: RawFd, buf: &'a [u8]) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as _).build();
        submit(sqe)?.await.map(|n| n as _)
    }
}

/// See `man pwrite.2` for more details.
pub fn pwrite<'a>(fd: RawFd, buf: &'a [u8], pos: u64) -> impl Future<Output = Result<usize>> + 'a {
    async move {
        let sqe = opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as _)
            .offset(pos as _)
            .build();
        submit(sqe)?.await.map(|n| n as _)
    }
}

/// See `man open.2` for more details.
pub fn open(
    path: &Path,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> impl Future<Output = Result<RawFd>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
            .flags(flags)
            .mode(mode)
            .build();
        submit(sqe)?.await.map(|fd| fd as _)
    }
}

/// See `man close.2` for more details.
#[allow(dead_code)]
pub fn close(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Close::new(types::Fd(fd)).build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man fstat.2` for more details.
pub fn fstat(fd: RawFd) -> impl Future<Output = Result<libc::statx>> {
    async move {
        let mut stat = unsafe { mem::zeroed() };
        let sqe = opcode::Statx::new(
            types::Fd(fd),
            std::ptr::null(),
            &mut stat as *mut _ as *mut _,
        )
        .mask(libc::STATX_ALL)
        .build();
        submit(sqe)?.await.map(|_| stat)
    }
}

/// See `man fsync.2` for more details.
pub fn fsync(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Fsync::new(types::Fd(fd)).build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man fdatasync.2` for more details.
pub fn fdatasync(fd: RawFd) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Fsync::new(types::Fd(fd))
            .flags(types::FsyncFlags::DATASYNC)
            .build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man mkdir.2` for more details.
pub fn mkdir(path: &Path, mode: libc::mode_t) -> impl Future<Output = Result<()>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe = opcode::MkDirAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
            .mode(mode)
            .build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man rmdir.2` for more details.
pub fn rmdir(path: &Path) -> impl Future<Output = Result<()>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe = opcode::UnlinkAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
            .flags(libc::AT_REMOVEDIR)
            .build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man unlink.2` for more details.
pub fn unlink(path: &Path) -> impl Future<Output = Result<()>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe =
            opcode::UnlinkAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr()).build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man rename.2` for more details.
pub fn rename(oldpath: &Path, newpath: &Path) -> impl Future<Output = Result<()>> {
    let oldpath = new_path_str(oldpath);
    let newpath = new_path_str(newpath);
    async move {
        let oldpath = oldpath?;
        let newpath = newpath?;
        let sqe = opcode::RenameAt::new(
            types::Fd(libc::AT_FDCWD),
            oldpath.as_c_str().as_ptr(),
            types::Fd(libc::AT_FDCWD),
            newpath.as_c_str().as_ptr(),
        )
        .build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man accept.2` for more details.
pub fn accept(fd: RawFd) -> impl Future<Output = Result<(RawFd, SockAddr)>> {
    async move {
        let mut addr = unsafe { mem::zeroed() };
        let mut addr_len = mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        let sqe = opcode::Accept::new(types::Fd(fd), &mut addr as *mut _ as *mut _, &mut addr_len)
            .build();
        let (_, sock_addr) = unsafe {
            SockAddr::init(|a, l| {
                *a = addr;
                *l = addr_len;
                Ok(())
            })
            .unwrap()
        };
        submit(sqe)?.await.map(|fd| (fd as _, sock_addr))
    }
}

/// See `man connect.2` for more details.
pub fn connect(fd: RawFd, addr: SockAddr) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Connect::new(types::Fd(fd), addr.as_ptr(), addr.len()).build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See `man shutdown.2` for more details.
pub fn shutdown(fd: RawFd, how: libc::c_int) -> impl Future<Output = Result<()>> {
    async move {
        let sqe = opcode::Shutdown::new(types::Fd(fd), how).build();
        submit(sqe)?.await.map(|_| ())
    }
}

fn new_path_str(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::from(ErrorKind::InvalidFilename))
}

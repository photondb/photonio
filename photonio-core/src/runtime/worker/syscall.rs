//! Asynchronous system calls.

use std::{
    ffi::CString,
    future::Future,
    io::{Error, ErrorKind, Result},
    mem,
    os::unix::{
        ffi::OsStrExt,
        io::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    },
    path::Path,
};

use io_uring::{opcode, types};
use socket2::SockAddr;

use super::submit;

/// See also `man open.2`.
pub fn open(
    path: &Path,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> impl Future<Output = Result<OwnedFd>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
            .flags(flags)
            .mode(mode)
            .build();
        submit(sqe)?
            .await
            .map(|fd| unsafe { OwnedFd::from_raw_fd(fd as _) })
    }
}

/// See also `man close.2`.
#[allow(dead_code)]
pub async fn close(fd: OwnedFd) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Close::new(fd).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man fstat.2`.
pub async fn fstat(fd: BorrowedFd<'_>) -> Result<libc::statx> {
    let fd = types::Fd(fd.as_raw_fd());
    let mut stat = unsafe { mem::zeroed() };
    let sqe = opcode::Statx::new(fd, std::ptr::null(), &mut stat as *mut _ as *mut _)
        .mask(libc::STATX_ALL)
        .build();
    submit(sqe)?.await.map(|_| stat)
}

/// See also `man fsync.2`.
pub async fn fsync(fd: BorrowedFd<'_>) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Fsync::new(fd).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man fdatasync.2`.
pub async fn fdatasync(fd: BorrowedFd<'_>) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Fsync::new(fd)
        .flags(types::FsyncFlags::DATASYNC)
        .build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man mkdir.2`.
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

/// See also `man rmdir.2`.
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

/// See also `man unlink.2`.
pub fn unlink(path: &Path) -> impl Future<Output = Result<()>> {
    let path = new_path_str(path);
    async move {
        let path = path?;
        let sqe =
            opcode::UnlinkAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr()).build();
        submit(sqe)?.await.map(|_| ())
    }
}

/// See also `man rename.2`.
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

/// See also `man accept.2`.
pub async fn accept(fd: BorrowedFd<'_>) -> Result<(OwnedFd, SockAddr)> {
    let fd = types::Fd(fd.as_raw_fd());
    let mut addr = unsafe { mem::zeroed() };
    let mut addr_len = mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    let sqe = opcode::Accept::new(fd, &mut addr as *mut _ as *mut _, &mut addr_len).build();
    let (_, sock_addr) = unsafe {
        SockAddr::init(|a, l| {
            *a = addr;
            *l = addr_len;
            Ok(())
        })
        .unwrap()
    };
    submit(sqe)?.await.map(|fd| {
        let conn = unsafe { OwnedFd::from_raw_fd(fd as _) };
        (conn, sock_addr)
    })
}

/// See also `man connect.2`.
pub async fn connect(fd: BorrowedFd<'_>, addr: SockAddr) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Connect::new(fd, addr.as_ptr(), addr.len()).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man shutdown.2`.
pub async fn shutdown(fd: BorrowedFd<'_>, how: libc::c_int) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Shutdown::new(fd, how).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man read.2`.
pub async fn read<'a>(fd: BorrowedFd<'a>, buf: &'a mut [u8]) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as _).build();
    submit(sqe)?.await.map(|n| n as _)
}

/// See also `man pread.2`.
pub async fn pread<'a>(fd: BorrowedFd<'a>, buf: &'a mut [u8], pos: u64) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as _)
        .offset(pos as _)
        .build();
    submit(sqe)?.await.map(|n| n as _)
}

/// See also `man write.2`.
pub async fn write<'a>(fd: BorrowedFd<'a>, buf: &'a [u8]) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Write::new(fd, buf.as_ptr(), buf.len() as _).build();
    submit(sqe)?.await.map(|n| n as _)
}

/// See also `man pwrite.2`.
pub async fn pwrite<'a>(fd: BorrowedFd<'a>, buf: &'a [u8], pos: u64) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Write::new(fd, buf.as_ptr(), buf.len() as _)
        .offset(pos as _)
        .build();
    submit(sqe)?.await.map(|n| n as _)
}

fn new_path_str(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::from(ErrorKind::InvalidFilename))
}

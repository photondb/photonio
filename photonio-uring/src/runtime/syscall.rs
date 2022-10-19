//! Asynchronous system calls.

use std::{
    ffi::CString,
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

use super::worker::submit;

/// See also `man open.2`.
pub(crate) async fn open(path: &Path, flags: libc::c_int, mode: libc::mode_t) -> Result<OwnedFd> {
    let path = new_path_str(path)?;
    let sqe = opcode::OpenAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
        .flags(flags | libc::O_CLOEXEC)
        .mode(mode)
        .build();
    submit(sqe)?
        .await
        .map(|fd| unsafe { OwnedFd::from_raw_fd(fd as _) })
}

/// See also `man close.2`.
#[allow(dead_code)]
pub(crate) async fn close(fd: OwnedFd) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Close::new(fd).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man fstat.2`.
pub(crate) async fn fstat(fd: BorrowedFd<'_>) -> Result<libc::statx> {
    let fd = types::Fd(fd.as_raw_fd());
    let path = CString::new("").unwrap();
    let mut stat = unsafe { mem::zeroed() };
    let sqe = opcode::Statx::new(fd, path.as_ptr(), &mut stat as *mut _ as *mut _)
        .flags(libc::AT_EMPTY_PATH)
        .mask(libc::STATX_ALL)
        .build();
    submit(sqe)?.await.map(|_| stat)
}

/// See also `man fsync.2`.
pub(crate) async fn fsync(fd: BorrowedFd<'_>) -> Result<()> {
    fsync_inner(fd, types::FsyncFlags::empty()).await
}

/// See also `man fdatasync.2`.
pub(crate) async fn fdatasync(fd: BorrowedFd<'_>) -> Result<()> {
    fsync_inner(fd, types::FsyncFlags::DATASYNC).await
}

async fn fsync_inner(fd: BorrowedFd<'_>, flags: types::FsyncFlags) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Fsync::new(fd).flags(flags).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man mkdir.2`.
pub(crate) async fn mkdir(path: &Path, mode: libc::mode_t) -> Result<()> {
    let path = new_path_str(path)?;
    let sqe = opcode::MkDirAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
        .mode(mode)
        .build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man rmdir.2`.
pub(crate) async fn rmdir(path: &Path) -> Result<()> {
    unlink_inner(path, libc::AT_REMOVEDIR).await
}

/// See also `man unlink.2`.
pub(crate) async fn unlink(path: &Path) -> Result<()> {
    unlink_inner(path, 0).await
}

async fn unlink_inner(path: &Path, flags: libc::c_int) -> Result<()> {
    let path = new_path_str(path)?;
    let sqe = opcode::UnlinkAt::new(types::Fd(libc::AT_FDCWD), path.as_c_str().as_ptr())
        .flags(flags)
        .build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man rename.2`.
pub(crate) async fn rename(oldpath: &Path, newpath: &Path) -> Result<()> {
    let oldpath = new_path_str(oldpath)?;
    let newpath = new_path_str(newpath)?;
    let sqe = opcode::RenameAt::new(
        types::Fd(libc::AT_FDCWD),
        oldpath.as_c_str().as_ptr(),
        types::Fd(libc::AT_FDCWD),
        newpath.as_c_str().as_ptr(),
    )
    .build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man accept.2`.
pub(crate) async fn accept(fd: BorrowedFd<'_>) -> Result<(OwnedFd, SockAddr)> {
    let fd = types::Fd(fd.as_raw_fd());
    let mut addr: libc::sockaddr_storage = unsafe { mem::zeroed() };
    let mut addr_len = mem::size_of_val(&addr) as libc::socklen_t;
    let sqe = opcode::Accept::new(fd, &mut addr as *mut _ as *mut _, &mut addr_len)
        .flags(libc::O_CLOEXEC)
        .build();
    let conn = submit(sqe)?.await?;
    unsafe {
        let conn = OwnedFd::from_raw_fd(conn as _);
        let sock_addr = SockAddr::new(addr, addr_len);
        Ok((conn, sock_addr))
    }
}

/// See also `man connect.2`.
pub(crate) async fn connect(fd: BorrowedFd<'_>, addr: SockAddr) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Connect::new(fd, addr.as_ptr(), addr.len()).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man shutdown.2`.
pub(crate) async fn shutdown(fd: BorrowedFd<'_>, how: libc::c_int) -> Result<()> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Shutdown::new(fd, how).build();
    submit(sqe)?.await.map(|_| ())
}

/// See also `man read.2`.
pub(crate) async fn read<'a>(fd: BorrowedFd<'a>, buf: &'a mut [u8]) -> Result<usize> {
    pread(fd, buf, -1).await
}

/// See also `man pread.2`.
pub(crate) async fn pread<'a>(
    fd: BorrowedFd<'a>,
    buf: &'a mut [u8],
    pos: libc::off64_t,
) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as _)
        .offset(pos)
        .build();
    submit(sqe)?.await.map(|n| n as _)
}

/// See also `man write.2`.
pub(crate) async fn write<'a>(fd: BorrowedFd<'a>, buf: &'a [u8]) -> Result<usize> {
    pwrite(fd, buf, -1).await
}

/// See also `man pwrite.2`.
pub(crate) async fn pwrite<'a>(
    fd: BorrowedFd<'a>,
    buf: &'a [u8],
    pos: libc::off64_t,
) -> Result<usize> {
    let fd = types::Fd(fd.as_raw_fd());
    let sqe = opcode::Write::new(fd, buf.as_ptr(), buf.len() as _)
        .offset(pos)
        .build();
    submit(sqe)?.await.map(|n| n as _)
}

fn new_path_str(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).map_err(|_| Error::from(ErrorKind::InvalidFilename))
}

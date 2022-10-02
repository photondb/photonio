use std::{
    future::Future,
    io::Result,
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
    path::Path,
};

use super::Metadata;
use crate::{
    io::{Read, ReadAt, Write, WriteAt},
    runtime::syscall,
};

/// A reference to an open file.
///
/// This type is an async version of [`std::fs::File`].
#[derive(Debug)]
pub struct File(OwnedFd);

impl File {
    /// See also [`std::fs::File::open`].
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new().read(true).open(path).await
    }

    /// See also [`std::fs::File::create`].
    pub async fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
    }

    /// See also [`std::fs::File::metadata`].
    pub async fn metadata(&self) -> Result<Metadata> {
        syscall::fstat(self.fd()).await.map(Metadata::new)
    }

    /// See also [`std::fs::File::sync_all`].
    pub async fn sync_all(&self) -> Result<()> {
        syscall::fsync(self.fd()).await
    }

    /// See also [`std::fs::File::sync_data`].
    pub async fn sync_data(&self) -> Result<()> {
        syscall::fdatasync(self.fd()).await
    }
}

impl File {
    fn fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsFd for File {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(OwnedFd::from_raw_fd(fd))
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Read for File {
    type Read<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a> {
        syscall::read(self.fd(), buf)
    }
}

impl ReadAt for File {
    type ReadAt<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read_at<'a>(&'a self, buf: &'a mut [u8], pos: u64) -> Self::ReadAt<'a> {
        syscall::pread(self.fd(), buf, pos)
    }
}

impl Write for File {
    type Write<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a> {
        syscall::write(self.fd(), buf)
    }
}

impl WriteAt for File {
    type WriteAt<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write_at<'a>(&'a self, buf: &'a [u8], pos: u64) -> Self::WriteAt<'a> {
        syscall::pwrite(self.0.as_fd(), buf, pos)
    }
}

/// Options to configure how a file is opened.
///
/// This type is an async version of [`std::fs::OpenOptions`].
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    /// See also [`std::fs::OpenOptions::new`].
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    /// See also [`std::fs::OpenOptions::read`].
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// See also [`std::fs::OpenOptions::write`].
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// See also [`std::fs::OpenOptions::append`].
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// See also [`std::fs::OpenOptions::truncate`].
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// See also [`std::fs::OpenOptions::create`].
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// See also [`std::fs::OpenOptions::create_new`].
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// See also [`std::fs::OpenOptions::open`].
    pub async fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let fd = syscall::open(path.as_ref(), self.open_flags(), 0o666).await?;
        Ok(File(fd))
    }
}

impl OpenOptions {
    fn open_flags(&self) -> libc::c_int {
        let mut flags = match (self.read, self.write, self.append) {
            (true, _, true) => libc::O_RDWR | libc::O_APPEND,
            (true, true, false) => libc::O_RDWR,
            (true, false, false) => libc::O_RDONLY,
            (false, _, true) => libc::O_WRONLY | libc::O_APPEND,
            (false, true, false) => libc::O_WRONLY,
            (false, false, false) => 0,
        };
        if self.create_new {
            flags |= libc::O_CREAT | libc::O_EXCL;
        } else {
            if self.create {
                flags |= libc::O_CREAT;
            }
            if self.truncate {
                flags |= libc::O_TRUNC;
            }
        }
        flags | libc::O_CLOEXEC
    }
}

use std::{
    future::Future,
    io::Result,
    os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    path::Path,
};

use super::Metadata;
use crate::io::{syscall, Read, ReadAt, Write, WriteAt};

/// A reference to an open file.
///
/// This type is an async version of [`std::fs::File`].
pub struct File(OwnedFd);

impl File {
    /// See [`std::fs::File::open`].
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new().read(true).open(path).await
    }

    /// See [`std::fs::File::metadata`].
    pub async fn metadata(&self) -> Result<Metadata> {
        syscall::fstat(self.raw_fd()).await.map(Metadata)
    }

    /// See [`std::fs::File::sync_all`].
    pub async fn sync_all(&self) -> Result<()> {
        syscall::fsync(self.raw_fd()).await
    }

    /// See [`std::fs::File::sync_data`].
    pub async fn sync_data(&self) -> Result<()> {
        syscall::fdatasync(self.raw_fd()).await
    }
}

impl File {
    fn raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Read for File {
    type Read<'b> = impl Future<Output = Result<usize>> + 'b;

    fn read<'b>(&mut self, buf: &'b mut [u8]) -> Self::Read<'b> {
        syscall::read(self.raw_fd(), buf)
    }
}

impl ReadAt for File {
    type ReadAt<'b> = impl Future<Output = Result<usize>> + 'b;

    fn read_at<'b>(&self, buf: &'b mut [u8], pos: u64) -> Self::ReadAt<'b> {
        syscall::pread(self.raw_fd(), buf, pos)
    }
}

impl Write for File {
    type Write<'b> = impl Future<Output = Result<usize>> + 'b;

    fn write<'b>(&mut self, buf: &'b [u8]) -> Self::Write<'b> {
        syscall::write(self.raw_fd(), buf)
    }
}

impl WriteAt for File {
    type WriteAt<'b> = impl Future<Output = Result<usize>> + 'b;

    fn write_at<'b>(&self, buf: &'b [u8], pos: u64) -> Self::WriteAt<'b> {
        syscall::pwrite(self.raw_fd(), buf, pos)
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
    /// See [`std::fs::OpenOptions::new`].
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

    /// See [`std::fs::OpenOptions::read`].
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// See [`std::fs::OpenOptions::write`].
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// See [`std::fs::OpenOptions::append`].
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// See [`std::fs::OpenOptions::truncate`].
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// See [`std::fs::OpenOptions::create`].
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// See [`std::fs::OpenOptions::create_new`].
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// See [`std::fs::OpenOptions::open`].
    pub async fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let fd = syscall::open(path.as_ref(), self.open_flags(), 0o666).await?;
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        Ok(File(owned_fd))
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

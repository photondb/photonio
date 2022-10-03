use std::{
    future::Future,
    io::Result,
    os::unix::io::{AsFd, BorrowedFd, OwnedFd},
    path::Path,
};

use super::{Metadata, OpenOptions};
use crate::{
    io::{Read, ReadAt, Write, WriteAt},
    runtime::syscall,
};

/// A reference to an open file.
///
/// This type is an async version of [`std::fs::File`].
#[derive(Debug)]
pub struct File(pub(super) OwnedFd);

impl File {
    /// Opens a file in read-only mode.
    ///
    /// See also [`std::fs::File::open`].
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new().read(true).open(path).await
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist, and will truncate it if it does.
    ///
    /// See also [`std::fs::File::create`].
    pub async fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await
    }

    /// Returns the metadata about this file.
    ///
    /// See also [`std::fs::File::metadata`].
    pub async fn metadata(&self) -> Result<Metadata> {
        syscall::fstat(self.fd()).await.map(Metadata::new)
    }

    /// Synchronizes all modified data of this file to disk.
    ///
    /// See also [`std::fs::File::sync_all`].
    pub async fn sync_all(&self) -> Result<()> {
        syscall::fsync(self.fd()).await
    }

    /// This function is similiar to [`sync_all`], except that it might not synchronize metadata.
    ///
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

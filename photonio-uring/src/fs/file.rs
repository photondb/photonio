use std::{
    future::{ready, Future},
    io::{Error, ErrorKind, Result, Seek as _},
    mem::ManuallyDrop,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
    path::Path,
};

use super::{Metadata, OpenOptions};
use crate::{
    io::{Read, ReadAt, Seek, SeekFrom, Write, WriteAt},
    runtime::syscall,
};

/// A reference to an open file.
///
/// This type is an async version of [`std::fs::File`].
#[derive(Debug)]
pub struct File(OwnedFd);

impl File {
    /// Opens a file in read-only mode.
    ///
    /// See also [`std::fs::File::open`].
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        OpenOptions::new().read(true).open(path).await
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist, and will truncate
    /// it if it does.
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
        syscall::fstat(self.as_fd()).await.map(Metadata::from)
    }

    /// Truncates or extends the size of this file.
    ///
    /// See also [`std::fs::File::set_len`].
    pub async fn set_len(&self, size: u64) -> Result<()> {
        self.as_std(|file| file.set_len(size))
    }

    /// Synchronizes all modified data of this file to disk.
    ///
    /// See also [`std::fs::File::sync_all`].
    pub async fn sync_all(&self) -> Result<()> {
        syscall::fsync(self.as_fd()).await
    }

    /// This function is similiar to [`Self::sync_all`], except that it might
    /// not synchronize metadata.
    ///
    /// See also [`std::fs::File::sync_data`].
    pub async fn sync_data(&self) -> Result<()> {
        syscall::fdatasync(self.as_fd()).await
    }
}

impl File {
    fn as_std<F, R>(&self, f: F) -> R
    where
        F: Fn(&mut std::fs::File) -> R,
    {
        // Convert the file to a `std::fs::File` without taking its ownership.
        let fd = self.0.as_raw_fd();
        let mut file = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(fd)) };
        f(&mut file)
    }
}

#[doc(hidden)]
impl From<OwnedFd> for File {
    fn from(fd: OwnedFd) -> Self {
        Self(fd)
    }
}

#[doc(hidden)]
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

impl Seek for File {
    type Seek = impl Future<Output = Result<u64>>;

    fn seek(&mut self, pos: SeekFrom) -> Self::Seek {
        ready(self.as_std(|file| file.seek(pos)))
    }
}

impl Read for File {
    type Read<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a> {
        syscall::read(self.0.as_fd(), buf)
    }
}

impl ReadAt for File {
    type ReadAt<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read_at<'a>(&'a self, buf: &'a mut [u8], pos: u64) -> Self::ReadAt<'a> {
        async move {
            let pos = pos
                .try_into()
                .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            syscall::pread(self.0.as_fd(), buf, pos).await
        }
    }
}

impl Write for File {
    type Write<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a> {
        syscall::write(self.0.as_fd(), buf)
    }
}

impl WriteAt for File {
    type WriteAt<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write_at<'a>(&'a self, buf: &'a [u8], pos: u64) -> Self::WriteAt<'a> {
        async move {
            let pos = pos
                .try_into()
                .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            syscall::pwrite(self.0.as_fd(), buf, pos).await
        }
    }
}

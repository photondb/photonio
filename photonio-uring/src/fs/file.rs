use std::{
    future::{ready, Future},
    io::Result,
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
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
        syscall::fstat(self.fd()).await.map(Metadata::from)
    }

    /// Synchronizes all modified data of this file to disk.
    ///
    /// See also [`std::fs::File::sync_all`].
    pub async fn sync_all(&self) -> Result<()> {
        syscall::fsync(self.fd()).await
    }

    /// This function is similiar to [`Self::sync_all`], except that it might not synchronize
    /// metadata.
    ///
    /// See also [`std::fs::File::sync_data`].
    pub async fn sync_data(&self) -> Result<()> {
        syscall::fdatasync(self.fd()).await
    }

    /// This function is similiar to [`Self::set_len`], except that it might not synchronize
    /// metadata.
    ///
    /// See also [`std::fs::File::set_len`].
    pub async fn set_len(&self, size: u64) -> Result<()> {
        use libc::*;
        fn cvt(t: libc::c_int) -> crate::io::Result<libc::c_int> {
            if t == -1 {
                Err(crate::io::Error::last_os_error())
            } else {
                Ok(t)
            }
        }

        fn cvt_r<F>(mut f: F) -> crate::io::Result<libc::c_int>
        where
            F: FnMut() -> libc::c_int,
        {
            loop {
                match cvt(f()) {
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    other => return other,
                }
            }
        }
        let size: off64_t = size
            .try_into()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        cvt_r(|| unsafe { ftruncate64(self.as_raw_fd(), size) }).map(drop)?;
        Ok(())
    }
}

impl File {
    fn fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[doc(hidden)]
impl From<OwnedFd> for File {
    fn from(fd: OwnedFd) -> Self {
        Self(fd)
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

impl Seek for File {
    type Seek = impl Future<Output = Result<u64>>;

    fn seek(&mut self, pos: SeekFrom) -> Self::Seek {
        let (offset, whence) = match pos {
            SeekFrom::Start(offset) => (offset as i64, libc::SEEK_SET),
            SeekFrom::End(offset) => (offset, libc::SEEK_END),
            SeekFrom::Current(offset) => (offset, libc::SEEK_CUR),
        };
        let ret = unsafe { libc::lseek(self.0.as_raw_fd(), offset, whence) };
        let res = if ret >= 0 {
            Ok(ret as u64)
        } else {
            Err(std::io::Error::last_os_error())
        };
        ready(res)
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
        syscall::pread(self.fd(), buf, pos.try_into().unwrap())
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
        syscall::pwrite(self.0.as_fd(), buf, pos.try_into().unwrap())
    }
}

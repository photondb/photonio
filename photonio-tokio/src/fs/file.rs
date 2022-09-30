use std::{future::Future, io::Result, mem::ManuallyDrop};

use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::Metadata;
use crate::io::{Read, ReadAt, Write, WriteAt};

pub struct File(fs::File);

impl File {
    pub async fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        fs::File::open(path).await.map(Self)
    }

    pub async fn create<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        fs::File::create(path).await.map(Self)
    }

    pub async fn metadata(&self) -> Result<Metadata> {
        self.0.metadata().await.map(Metadata)
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.0.sync_all().await
    }

    pub async fn sync_data(&self) -> Result<()> {
        self.0.sync_data().await
    }
}

impl Read for File {
    type Read<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Read<'a> {
        self.0.read(buf)
    }
}

#[cfg(unix)]
impl ReadAt for File {
    type ReadAt<'a> = impl Future<Output = Result<usize>> + 'a;

    // FIXME: Makes it asynchronous when Tokio supports positional reads.
    fn read_at<'a>(&'a self, buf: &'a mut [u8], pos: u64) -> Self::ReadAt<'a> {
        use std::os::unix::{
            fs::FileExt,
            io::{AsRawFd, FromRawFd},
        };

        let file = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(self.0.as_raw_fd())) };
        async move { file.read_at(buf, pos) }
    }
}

impl Write for File {
    type Write<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Write<'a> {
        self.0.write(buf)
    }
}

#[cfg(unix)]
impl WriteAt for File {
    type WriteAt<'a> = impl Future<Output = Result<usize>> + 'a;

    // FIXME: Makes it asynchronous when Tokio supports positional writes.
    fn write_at<'a>(&'a self, buf: &'a [u8], pos: u64) -> Self::WriteAt<'a> {
        use std::os::unix::{
            fs::FileExt,
            io::{AsRawFd, FromRawFd},
        };

        let file = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(self.0.as_raw_fd())) };
        async move { file.write_at(buf, pos) }
    }
}

use std::{future::Future, io::Result, os::unix::io::RawFd, path::Path};

use crate::io::{op, Read, Write};

pub struct File(RawFd);

impl File {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new().read(true).open(path).await
    }

    pub async fn metadata(&self) -> Result<Metadata> {
        op::statx(self.0).await.map(Metadata::from)
    }

    pub async fn sync_all(&self) -> Result<()> {
        op::sync_all(self.0).await
    }

    pub async fn sync_data(&self) -> Result<()> {
        op::sync_data(self.0).await
    }
}

impl Read for File {
    type ReadFuture<'a> = impl Future<Output = Result<usize>> + 'a;

    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        op::read(self.0, buf)
    }
}

impl Write for File {
    type WriteFuture<'a> = impl Future<Output = Result<usize>> + 'a;

    fn write<'a>(&mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        op::write(self.0, buf)
    }
}

pub struct Metadata {
    len: u64,
}

impl Metadata {
    pub fn len(&self) -> u64 {
        self.len
    }
}

impl From<libc::statx> for Metadata {
    fn from(stat: libc::statx) -> Self {
        Self { len: stat.stx_size }
    }
}

pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    creation_mode: u32,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            creation_mode: 0o666,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn creation_mode(&mut self, creation_mode: u32) -> &mut Self {
        self.creation_mode = creation_mode;
        self
    }

    pub async fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        let fd = op::open(path.as_ref(), self.open_flags(), self.creation_mode).await?;
        Ok(File(fd))
    }

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
        flags
    }
}

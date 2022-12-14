use std::{io::Result, os::unix::fs::OpenOptionsExt, path::Path};

use super::File;
use crate::runtime::syscall;

/// Options to configure how a file is opened.
///
/// This type is an async version of [`std::fs::OpenOptions`].
#[derive(Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    mode: u32,
    custom_flags: i32,
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
            mode: 0o666,
            custom_flags: 0,
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
        let path = path.as_ref();
        syscall::open(path, self.flags(), self.mode)
            .await
            .map(File::from)
    }
}

impl OpenOptions {
    fn flags(&self) -> libc::c_int {
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
        flags |= self.custom_flags & !libc::O_ACCMODE;
        flags
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenOptionsExt for OpenOptions {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }
}

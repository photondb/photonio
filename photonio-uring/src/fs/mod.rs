//! Primitives for asynchronous filesystem operations.
//!
//! This module is an async version of [`std::fs`].

use std::{io::Result, path::Path};

use crate::runtime::syscall;

mod open;
pub use open::OpenOptions;

mod file;
pub use file::File;

mod metadata;
pub use metadata::Metadata;

/// An async version of [`std::fs::rename`].
pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    syscall::rename(from, to).await
}

/// An async version of [`std::fs::remove_file`].
pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    syscall::unlink(path).await
}

/// An async version of [`std::fs::create_dir`].
pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    syscall::mkdir(path, 0o777).await
}

/// An async version of [`std::fs::remove_dir`].
pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    syscall::rmdir(path).await
}

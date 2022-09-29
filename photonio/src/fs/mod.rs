//! Filesystem types and operations.
//!
//! This module is an async version of [`std::fs`].

use std::{io::Result, path::Path};

use crate::io::op;

mod file;
pub use file::{File, Metadata, OpenOptions};

/// This function is an async version of [`std::fs::rename`].
pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    op::rename(from.as_ref(), to.as_ref()).await
}

/// This function is an async version of [`std::fs::remove_file`].
pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    op::unlink(path.as_ref()).await
}

/// This function is an async version of [`std::fs::create_dir`].
pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    op::mkdir(path.as_ref(), 0o777).await
}

/// This function is an async version of [`std::fs::remove_dir`].
pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    op::rmdir(path.as_ref()).await
}

use std::{io::Result, path::Path};

use crate::io::op;

mod file;
pub use file::{File, OpenOptions};

pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    op::rename(from.as_ref(), to.as_ref()).await
}

pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    op::unlink(path.as_ref()).await
}

pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    op::mkdir(path.as_ref(), 0o777).await
}

pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    op::rmdir(path.as_ref()).await
}

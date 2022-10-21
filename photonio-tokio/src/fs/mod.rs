use std::{io::Result, path::Path};

mod open;
pub use open::OpenOptions;

mod file;
pub use file::File;

mod metadata;
pub use metadata::Metadata;

pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    tokio::fs::rename(from, to).await
}

pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    tokio::fs::remove_file(path).await
}

pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    tokio::fs::create_dir(path).await
}

pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    tokio::fs::remove_dir(path).await
}

use std::{io::Result, path::Path};

use tokio::fs;

use super::File;

pub struct OpenOptions(fs::OpenOptions);

impl OpenOptions {
    pub fn new() -> Self {
        Self(fs::OpenOptions::new())
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.0.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.0.write(write);
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.0.append(append);
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.0.truncate(truncate);
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.0.create(create);
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.0.create_new(create_new);
        self
    }

    pub async fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        self.0.open(path).await.map(File::from)
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<fs::OpenOptions> for OpenOptions {
    fn from(options: fs::OpenOptions) -> Self {
        Self(options)
    }
}

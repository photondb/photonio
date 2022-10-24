use std::fs;

#[derive(Clone, Debug)]
pub struct Metadata(fs::Metadata);

impl Metadata {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.0.len()
    }

    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.0.is_symlink()
    }
}

impl From<fs::Metadata> for Metadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self(metadata)
    }
}

impl std::os::unix::prelude::MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        self.0.dev()
    }

    fn ino(&self) -> u64 {
        unimplemented!()
    }

    fn mode(&self) -> u32 {
        unimplemented!()
    }

    fn nlink(&self) -> u64 {
        unimplemented!()
    }

    fn uid(&self) -> u32 {
        unimplemented!()
    }

    fn gid(&self) -> u32 {
        unimplemented!()
    }

    fn rdev(&self) -> u64 {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        unimplemented!()
    }

    fn atime(&self) -> i64 {
        unimplemented!()
    }

    fn atime_nsec(&self) -> i64 {
        unimplemented!()
    }

    fn mtime(&self) -> i64 {
        unimplemented!()
    }

    fn mtime_nsec(&self) -> i64 {
        unimplemented!()
    }

    fn ctime(&self) -> i64 {
        unimplemented!()
    }

    fn ctime_nsec(&self) -> i64 {
        unimplemented!()
    }

    fn blksize(&self) -> u64 {
        unimplemented!()
    }

    fn blocks(&self) -> u64 {
        unimplemented!()
    }
}

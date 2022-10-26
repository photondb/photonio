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

#[cfg(unix)]
impl std::os::unix::fs::MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        self.0.dev()
    }

    fn ino(&self) -> u64 {
        self.0.ino()
    }

    fn mode(&self) -> u32 {
        self.0.mode()
    }

    fn nlink(&self) -> u64 {
        self.0.nlink()
    }

    fn uid(&self) -> u32 {
        self.0.uid()
    }

    fn gid(&self) -> u32 {
        self.0.gid()
    }

    fn rdev(&self) -> u64 {
        self.0.rdev()
    }

    fn size(&self) -> u64 {
        self.0.size()
    }

    fn atime(&self) -> i64 {
        self.0.atime()
    }

    fn atime_nsec(&self) -> i64 {
        self.0.atime_nsec()
    }

    fn mtime(&self) -> i64 {
        self.0.mtime()
    }

    fn mtime_nsec(&self) -> i64 {
        self.0.mtime_nsec()
    }

    fn ctime(&self) -> i64 {
        self.0.ctime()
    }

    fn ctime_nsec(&self) -> i64 {
        self.0.ctime_nsec()
    }

    fn blksize(&self) -> u64 {
        self.0.blksize()
    }

    fn blocks(&self) -> u64 {
        self.0.blocks()
    }
}

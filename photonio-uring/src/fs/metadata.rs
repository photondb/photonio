use std::fmt;

/// Metadata information about a file.
///
/// See also [`std::fs::Metadata`].
#[derive(Clone)]
pub struct Metadata(libc::statx);

impl Metadata {
    /// Returns the size of the file this metadata is for.
    ///
    /// See also [`std::fs::Metadata::len`].
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.0.stx_size
    }

    /// Returns true if this metadata is for a directory.
    ///
    /// See also [`std::fs::Metadata::is_dir`].
    pub fn is_dir(&self) -> bool {
        self.is_type(libc::S_IFDIR)
    }

    /// Returns true if this metadata is for a regular file.
    ///
    /// See also [`std::fs::Metadata::is_file`].
    pub fn is_file(&self) -> bool {
        self.is_type(libc::S_IFREG)
    }

    /// Returns true if this metadata is for a symbolic link.
    ///
    /// See also [`std::fs::Metadata::is_symlink`].
    pub fn is_symlink(&self) -> bool {
        self.is_type(libc::S_IFLNK)
    }
}

impl Metadata {
    fn is_type(&self, ty: libc::mode_t) -> bool {
        (self.0.stx_mode as u32 & libc::S_IFMT) == ty
    }
}

#[doc(hidden)]
impl From<libc::statx> for Metadata {
    fn from(stat: libc::statx) -> Self {
        Self(stat)
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("len", &self.len())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("is_symlink", &self.is_symlink())
            .finish()
    }
}

impl std::os::unix::fs::MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        libc::makedev(self.0.stx_dev_major, self.0.stx_dev_minor)
    }

    fn ino(&self) -> u64 {
        self.0.stx_ino
    }

    fn mode(&self) -> u32 {
        self.0.stx_mode.into()
    }

    fn nlink(&self) -> u64 {
        self.0.stx_nlink.into()
    }

    fn uid(&self) -> u32 {
        self.0.stx_uid
    }

    fn gid(&self) -> u32 {
        self.0.stx_gid
    }

    fn rdev(&self) -> u64 {
        libc::makedev(self.0.stx_rdev_major, self.0.stx_rdev_minor)
    }

    fn size(&self) -> u64 {
        self.0.stx_size
    }

    fn atime(&self) -> i64 {
        self.0.stx_atime.tv_sec
    }

    fn atime_nsec(&self) -> i64 {
        self.0.stx_atime.tv_nsec.into()
    }

    fn mtime(&self) -> i64 {
        self.0.stx_mtime.tv_sec
    }

    fn mtime_nsec(&self) -> i64 {
        self.0.stx_mtime.tv_nsec.into()
    }

    fn ctime(&self) -> i64 {
        self.0.stx_ctime.tv_sec
    }

    fn ctime_nsec(&self) -> i64 {
        self.0.stx_ctime.tv_nsec.into()
    }

    fn blksize(&self) -> u64 {
        self.0.stx_blksize.into()
    }

    fn blocks(&self) -> u64 {
        self.0.stx_blocks
    }
}

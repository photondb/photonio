use std::fmt;

/// Metadata information about a file.
///
/// See also [`std::fs::Metadata`].
#[derive(Clone)]
pub struct Metadata(libc::statx);

impl Metadata {
    pub(super) fn new(stat: libc::statx) -> Self {
        Self(stat)
    }

    /// See also [`std::fs::Metadata::len`].
    pub fn len(&self) -> u64 {
        self.0.stx_size
    }

    /// See also [`std::fs::Metadata::is_dir`].
    pub fn is_dir(&self) -> bool {
        self.is_type(libc::S_IFDIR)
    }

    /// See also [`std::fs::Metadata::is_file`].
    pub fn is_file(&self) -> bool {
        self.is_type(libc::S_IFREG)
    }

    /// See also [`std::fs::Metadata::is_symlink`].
    pub fn is_symlink(&self) -> bool {
        self.is_type(libc::S_IFLNK)
    }
}

impl Metadata {
    fn is_type(&self, ty: libc::mode_t) -> bool {
        if (self.0.stx_mode as u32 & libc::S_IFMT) == ty {
            true
        } else {
            false
        }
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

use std::fs;

#[derive(Clone, Debug)]
pub struct Metadata(pub(super) fs::Metadata);

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

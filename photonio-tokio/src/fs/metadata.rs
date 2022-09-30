pub struct Metadata(pub(super) std::fs::Metadata);

impl Metadata {
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
        self.0.file_type().is_symlink()
    }
}

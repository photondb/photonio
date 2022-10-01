pub struct Metadata(std::fs::Metadata);

impl Metadata {
    pub(super) fn new(metadata: std::fs::Metadata) -> Self {
        Self(metadata)
    }

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

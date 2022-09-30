use tokio::fs;

pub struct File(fs::File);

pub struct Metadata(std::fs::Metadata);

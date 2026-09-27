use std::time::SystemTime;

pub struct FileMetadata {
    pub len: u64,
    pub modified: Option<SystemTime>,
}

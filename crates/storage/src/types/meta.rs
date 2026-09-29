use std::time::SystemTime;

pub struct FileMetadata {
    len: u64,
    modified: Option<SystemTime>,
}

impl FileMetadata {
    /// Creates a new [`FileMetadata`]
    pub fn new(len: u64, modified: Option<SystemTime>) -> Self {
        Self { len, modified }
    }

    /// Returns the length of the file
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the file modification time
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }
}

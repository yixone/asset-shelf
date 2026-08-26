use crate::backend::StorageBackend;

/// The part of the storage system responsible for storing persistent files
#[derive(Debug)]
pub(crate) struct GlobalSection {
    /// File storage backend
    pub backend: Box<dyn StorageBackend>,
}

/// Data for global path generation
pub struct GlobalPathData<'a> {
    /// Container name
    pub(crate) container: &'a str,
    /// File path within the container
    pub(crate) file: &'a str,
}

impl<'a> GlobalPathData<'a> {
    /// Creates a new [`GlobalPathData`]
    pub fn new(container: &'a str, file: &'a str) -> Self {
        Self { container, file }
    }
}

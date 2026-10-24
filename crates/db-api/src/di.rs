use crate::ports::{AssetsDatabase, FilesDatabase};

/// Database data access context
pub struct DatabaseContext {
    pub assets: Box<dyn AssetsDatabase>,
    pub files: Box<dyn FilesDatabase>,
}

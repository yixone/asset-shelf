use crate::ports::{AssetDatabase, FilesDatabase};

/// Database data access context
pub struct DatabaseContext {
    pub assets: Box<dyn AssetDatabase>,
    pub files: Box<dyn FilesDatabase>,
}

use crate::{
    entities::{File, FileGroup},
    id::FileGroupId,
    patches::FilePatch,
    ports::UnitOfWork,
    result::Result,
    types::FileKey,
};

/// Provides access to persisted [`File`] data
#[async_trait::async_trait]
pub trait FileDatabase {
    /// Loads a [`File`] by its key
    ///
    /// Returns `None` if the file does not exist
    async fn get_file(&self, key: &FileKey) -> Result<Option<File>>;

    /// Updates a [`File`] using the given patch
    ///
    /// Returns `true` if the file was updated, or `false` if it does not exist
    async fn update_file(&self, key: &FileKey, patch: &FilePatch) -> Result<bool>;

    /// Deletes a [`File`]
    ///
    /// Returns `true` if the file was deleted, or `false` if it does not exist
    async fn delete_file(&self, key: &FileKey) -> Result<bool>;
}

/// Provides access to persisted [`FileGroup`] data
#[async_trait::async_trait]
pub trait FileGroupDatabase {
    /// Loads a [`FileGroup`] by its identifier
    ///
    /// Returns `None` if the file group does not exist
    async fn get_file_group(&self, id: &FileGroupId) -> Result<Option<FileGroup>>;

    /// Deletes the [`FileGroup`] and its related files
    ///
    /// Returns `true` if the file group was deleted, or `false` if it does not exist
    async fn delete_file_group(&self, id: &FileGroupId) -> Result<bool>;
}

/// Provides operations for creating persisted [`FileGroup`]s
#[async_trait::async_trait]
pub trait FileGroupWriter: UnitOfWork {
    /// Persists a new [`FileGroup`]
    async fn insert_file_group(&mut self, group: &FileGroup) -> Result<()>;
}

/// Provides operations for creating persisted [`File`]s
#[async_trait::async_trait]
pub trait FileWriter: UnitOfWork {
    /// Persist a new [`File`]
    async fn insert_file(&mut self, file: &File) -> Result<()>;
}

use crate::{
    Result,
    entities::{File, FileGroup},
    id::FileGroupId,
    patches::FilePatch,
    types::FileKey,
};

/// Provides access to persisted [`File`] data
#[async_trait::async_trait]
pub trait FileDatabase {
    /// Persist a new [`File`]
    async fn insert_file(&mut self, file: &File) -> Result<()>;

    /// Loads a [`File`] by its key
    ///
    /// Returns `None` if the file does not exist
    async fn get_file(&mut self, key: &FileKey) -> Result<Option<File>>;

    /// Updates a [`File`] using the given patch
    ///
    /// Returns the updated [`File`], or an error if the file does not
    /// exist or the update failes
    async fn update_file(&mut self, key: &FileKey, patch: &FilePatch) -> Result<File>;

    /// Deletes a [`File`]
    ///
    /// Returns `true` if the file was deleted, or `false` if
    /// it does not exist for the given key
    async fn delete_file(&mut self, key: &FileKey) -> Result<bool>;
}

/// Provides access to persisted [`FileGroup`] data
#[async_trait::async_trait]
pub trait FileGroupDatabase {
    /// Persists a new [`FileGroup`]
    async fn insert_file_group(&mut self, group: &FileGroup) -> Result<()>;

    /// Loads a [`FileGroup`] by its identifier
    ///
    /// Returns `None` if the file group does not exist
    async fn get_file_group(&mut self, id: &FileGroupId) -> Result<Option<FileGroup>>;

    /// Deletes the [`FileGroup`] and its related files
    ///
    /// Returns `true` if the file group was deleted, or `false` if it does not exist
    async fn delete_file_group(&mut self, id: &FileGroupId) -> Result<bool>;
}

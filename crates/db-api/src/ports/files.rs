use models::{
    entities::{File, FileGroup},
    id::FileGroupId,
    patches::FilePatch,
    types::FileKey,
};

use crate::{
    Result,
    ports::{DeleteFn, UpdateFn},
};

/// Database operations for files
#[async_trait::async_trait]
pub trait FilesDatabase: Send + Sync {
    /// Inserts a [`File`] into an existing [`FileGroup`]
    async fn insert_file(&self, file: &File) -> Result<()>;

    /// Retrieves a [`FileGroup`] by its ID
    ///
    /// Returns `None` if the file group does not exist
    async fn get_file_group_by_id(&self, id: &FileGroupId) -> Result<Option<FileGroup>>;

    /// Retrieves a [`File`] by its key
    ///
    /// Returns `None` if the file does not exist
    async fn get_file_by_key(&self, key: &FileKey) -> Result<Option<File>>;

    /// Loads a [`File`], applies a callback, and persists the modified entity
    ///
    /// The callback is executed against the current file state as part of
    /// the same atomic operation as persistence. If the callback returns an
    /// error, the update is aborted and no changes are persisted
    ///
    /// Returns the updated file, or an error if the file does not exist,
    /// validation fails, or persistence fails
    async fn update_file_fn_boxed(&self, key: &FileKey, f: Box<UpdateFn<File>>) -> Result<File>;

    /// Applies a patch to a [`File`] and persists the result
    async fn update_file(&self, key: &FileKey, patch: FilePatch) -> Result<File> {
        self.update_file_fn_boxed(
            key,
            Box::new(|f| {
                f.apply_patch(patch);
                Ok(())
            }),
        )
        .await
    }

    /// Loads a [`File`], validates whether it can be deleted, and deletes it
    ///
    /// The callback and deletion are performed as part of the same atomic operation.
    /// If the callback returns an error, the deletion is aborted
    ///
    /// Returns an error if the file does not exist, validation fails,
    /// or deletion fails
    async fn delete_file_fn_boxed(&self, key: &FileKey, f: Box<DeleteFn<File>>) -> Result<()>;

    /// Deletes a [`File`]
    async fn delete_file(&self, key: &FileKey) -> Result<()> {
        self.delete_file_fn_boxed(key, Box::new(|_| Ok(()))).await
    }

    /// Loads a [`FileGroup`], validates whether it can be deleted, and deletes it
    /// together with all its associated files
    ///
    /// The callback and deletion are performed as part of the same atomic operation.
    /// If the callback returns an error, the deletion is aborted
    ///
    /// Returns an error if the file group does not exist, validation fails,
    /// or deletion fails
    async fn delete_file_group_fn_boxed(
        &self,
        id: &FileGroupId,
        f: Box<DeleteFn<FileGroup>>,
    ) -> Result<()>;

    /// Deletes a [`FileGroup`] and all its associated files
    async fn delete_file_group(&self, id: &FileGroupId) -> Result<()> {
        self.delete_file_group_fn_boxed(id, Box::new(|_| Ok(())))
            .await
    }
}

#[async_trait::async_trait]
pub trait FilesDatabaseExt: FilesDatabase {
    /// Updates the [`File`] using an [`UpdateFn`] callback
    ///
    /// See [`FilesDatabase::update_file_fn_boxed`] for the operation's guarantees
    async fn update_file_fn<Fn>(&self, key: &FileKey, f: Fn) -> Result<File>
    where
        Fn: FnOnce(&mut File) -> std::result::Result<(), models::EntityError> + Send + 'static,
    {
        self.update_file_fn_boxed(key, Box::new(f)).await
    }

    /// Deletes a [`File`] after validating it with a [`DeleteFn`] callback
    ///
    /// See [`FilesDatabase::delete_file_fn_boxed`] for the operation's guarantees
    async fn delete_file_fn<Fn>(&self, key: &FileKey, f: Fn) -> Result<()>
    where
        Fn: FnOnce(&File) -> std::result::Result<(), models::EntityError> + Send + 'static,
    {
        self.delete_file_fn_boxed(key, Box::new(f)).await
    }

    /// Deletes a [`FileGroup`] after validating it with a [`DeleteFn`] callback
    ///
    /// See [`FilesDatabase::delete_file_group_fn_boxed`] for the operation's guarantees
    async fn delete_file_group_fn<Fn>(&self, id: &FileGroupId, f: Fn) -> Result<()>
    where
        Fn: FnOnce(&FileGroup) -> std::result::Result<(), models::EntityError> + Send + 'static,
    {
        self.delete_file_group_fn_boxed(id, Box::new(f)).await
    }
}

impl<T> FilesDatabaseExt for T where T: FilesDatabase {}

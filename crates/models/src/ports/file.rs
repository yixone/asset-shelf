use crate::{
    entities::{File, FileGroup},
    id::FileGroupId,
    patches::FilePatch,
    result::Result,
    types::FileKey,
};

#[async_trait::async_trait]
pub trait FileDatabase {
    async fn get_file(&self, key: &FileKey) -> Result<Option<File>>;

    async fn insert_file(&self, file: &File) -> Result<()>;

    async fn update_file(&self, key: &FileKey, patch: &FilePatch) -> Result<bool>;

    async fn delete_file(&self, key: &FileKey) -> Result<bool>;
}

#[async_trait::async_trait]
pub trait FileGroupDatabase {
    async fn get_file_group(&self, id: &FileGroupId) -> Result<Option<FileGroup>>;

    async fn insert_file_group(&self, group: &FileGroup) -> Result<()>;
    async fn insert_file_group_with_files(&self, group: &FileGroup) -> Result<()>;

    async fn delete_file_group(&self, id: &FileGroupId) -> Result<bool>;
}

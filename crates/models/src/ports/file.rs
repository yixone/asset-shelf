use crate::{
    entities::{File, file::FileGroup},
    id::FileGroupId,
    result::Result,
    types::FileVariant,
};

#[async_trait::async_trait]
pub trait FileDatabase {
    async fn insert_file(&self, file: &File) -> Result<()>;
}

#[async_trait::async_trait]
pub trait FileGroupDatabase {
    async fn get_file_group(&self, id: &FileGroupId) -> Result<Option<FileGroup>>;

    async fn get_file_group_variant(
        &self,
        id: &FileGroupId,
        variant: FileVariant,
    ) -> Result<Option<File>>;

    async fn insert_file_group_cascade(&self, group: &FileGroup) -> Result<()>;

    async fn insert_file_group(&self, group: &FileGroup) -> Result<()>;

    async fn delete_file_group(&self, id: &FileGroupId) -> Result<()>;
}

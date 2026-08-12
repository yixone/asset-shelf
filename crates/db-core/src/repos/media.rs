use models::{
    entities::{Media, MediaFile, MediaVariant},
    types::{MediaFileId, MediaId},
};
use result::Result;

use crate::{
    queries::media::MediaQuery,
    types::{DeleteResult, InsertResult, UpdateResult, patch::MediaFilePatch},
};

#[async_trait::async_trait]
pub trait MediaRepository: Send + Sync {
    async fn insert(&self, media: &Media) -> Result<InsertResult>;

    async fn insert_file(&self, file: &MediaFile) -> Result<InsertResult>;

    async fn update_file(
        &self,
        id: &MediaFileId,
        patch: MediaFilePatch,
    ) -> Result<UpdateResult<MediaFile>>;

    async fn delete_file(&self, id: &MediaFileId) -> Result<DeleteResult>;

    async fn get_variant(&self, id: &MediaId, variant: MediaVariant) -> Result<MediaFile>;

    async fn delete(&self, id: &MediaId) -> Result<DeleteResult>;

    async fn get_orphans(&self, limit: u32) -> Result<Vec<MediaQuery>>;
}

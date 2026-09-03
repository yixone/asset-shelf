use models::{
    media::{Media, MediaFile, MediaVariant, view::MediaView},
    types::{MediaFileId, MediaId},
};
use result::{Result, create_error};

use crate::types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::MediaFilePatch};

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

    async fn get_by_id(&self, id: &MediaId) -> Result<MediaView> {
        let q = self.get_by_ids(std::slice::from_ref(id)).await?;
        q.into_iter().next().ok_or(create_error!(NotFound))
    }

    async fn get_by_ids(&self, ids: &[MediaId]) -> Result<Vec<MediaView>>;

    async fn get_orphans(&self, limit: u32) -> Result<Vec<MediaView>>;

    async fn list_files(
        &self,
        pagination: Pagination,
        kind: MediaVariant,
    ) -> Result<Vec<MediaFile>>;
}

use models::{
    entities::{Media, MediaFile, MediaVariant},
    types::{MediaFileId, MediaId},
};
use result::Result;

use crate::types::{DeleteResult, InsertResult, UpdateResult, patches::MediaFilePatch};

pub trait MediaOps {
    /// Inserts a [`Media`] into the database
    async fn insert_media(&mut self, m: &Media) -> Result<InsertResult>;

    /// Removes [`Media`] from the database
    async fn delete_media(&mut self, id: &MediaId) -> Result<DeleteResult>;

    /// Returns a list of [`Media`] that are not referenced by anyone
    async fn get_orphans_media(&mut self, limit: u32) -> Result<Vec<Media>>;

    /// Returns the [`Media`] with the specified ID
    async fn get_media(&mut self, id: &MediaId) -> Result<Option<Media>> {
        self.get_media_bulk(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }
    /// Returns a set of [`Media`] objects based on a set of IDs
    async fn get_media_bulk(&mut self, ids: &[MediaId]) -> Result<Vec<Media>>;
}

pub trait MediaFilesOps {
    /// Inserts a [`MediaFile`] into the database
    async fn insert_media_file(&mut self, mf: &MediaFile) -> Result<InsertResult> {
        self.insert_media_file_bulk(std::slice::from_ref(mf)).await
    }

    /// Inserts a set of [`MediaFile`] into the database
    async fn insert_media_file_bulk(&mut self, mf: &[MediaFile]) -> Result<InsertResult>;

    async fn update_media_file(
        &mut self,
        id: &MediaFileId,
        patch: MediaFilePatch,
    ) -> Result<UpdateResult<MediaFile>>;

    /// Removes the [`MediaFile`] from the database.
    async fn delete_media_file(&mut self, id: &MediaFileId) -> Result<DeleteResult> {
        self.delete_media_file_bulk(std::slice::from_ref(id)).await
    }
    /// Removes the [`MediaFile`] from the database based on a set of IDs
    async fn delete_media_file_bulk(&mut self, ids: &[MediaFileId]) -> Result<DeleteResult>;

    /// Returns a [`MediaFile`] by its ID
    async fn get_media_file(&mut self, id: &MediaFileId) -> Result<Option<MediaFile>> {
        self.get_media_file_bulk(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }
    /// Returns a set of [`MediaFile`] objects based on a set of IDs
    async fn get_media_file_bulk(&mut self, ids: &[MediaFileId]) -> Result<Vec<MediaFile>>;

    /// Returns a set of [`MediaFile`] objects for the [`Media`] with the specified ID
    async fn get_media_files(&mut self, media_id: &MediaId) -> Result<Vec<MediaFile>> {
        self.get_media_files_bulk(std::slice::from_ref(media_id))
            .await
    }
    /// Returns a set of [`MediaFile`] objects for the set of [`Media`]
    async fn get_media_files_bulk(&mut self, media_ids: &[MediaId]) -> Result<Vec<MediaFile>>;

    /// Returns a [`MediaFile`] for [`Media`] with the specified variant
    async fn get_media_variant(
        &mut self,
        media_id: &MediaId,
        variant: MediaVariant,
    ) -> Result<Option<MediaFile>>;

    /// Counts the number of [`MediaFile`]s for the specified [`Media`]
    async fn count_media_files(&mut self, media_id: &MediaId) -> Result<i64>;
}

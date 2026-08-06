use models::{
    entities::{MediaFile, MediaVariant},
    types::{MediaFileId, MediaId},
};
use result::Result;

use crate::types::{DeleteResult, InsertResult, UpdateResult, patch::MediaFilePatch};

/// Read operations for data associated with the [`MediaFile`] domain
pub trait MediaFilesReadOps {
    /// Returns [`MediaFile`] for a given [`MediaFileId`]
    async fn get_media_file_by_id(&mut self, id: &MediaFileId) -> Result<Option<MediaFile>> {
        self.get_media_files_by_ids(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [`MediaFile`] objects based on a set of IDs
    async fn get_media_files_by_ids(&mut self, ids: &[MediaFileId]) -> Result<Vec<MediaFile>>;

    /// Returns a set of [`MediaFile`] objects for the [`Media`] with the specified ID
    async fn get_media_files_by_group(&mut self, media_id: &MediaId) -> Result<Vec<MediaFile>> {
        self.get_media_files_by_groups(std::slice::from_ref(media_id))
            .await
    }

    /// Returns a set of [`MediaFile`] objects for the set of [`Media`]
    async fn get_media_files_by_groups(&mut self, media_ids: &[MediaId]) -> Result<Vec<MediaFile>>;

    /// Returns a [`MediaFile`] for [`Media`] with the specified variant
    async fn get_media_variant(
        &mut self,
        media_id: &MediaId,
        variant: MediaVariant,
    ) -> Result<Option<MediaFile>>;
}

/// Write operations for data associated with the [`MediaFile`] domain
pub trait MediaFilesWriteOps {
    /// Inserts a [`MediaFile`] into the database
    async fn insert_media_file(&mut self, mf: &MediaFile) -> Result<InsertResult> {
        self.insert_many_media_files(std::slice::from_ref(mf)).await
    }

    /// Inserts a set of [`MediaFile`] into the database
    async fn insert_many_media_files(&mut self, mf: &[MediaFile]) -> Result<InsertResult>;

    /// Updates the [`MediaFile`] record with the
    /// specified ID according to the provided [`MediaFilePatch`]
    async fn update_media_file(
        &mut self,
        id: &MediaFileId,
        patch: MediaFilePatch,
    ) -> Result<UpdateResult<MediaFile>>;

    /// Removes the [`MediaFile`] from the database based on a set of IDs
    async fn delete_many_media_files(&mut self, ids: &[MediaFileId]) -> Result<DeleteResult>;
}

/// Operations on data associated with the [`MediaFile`] domain, used for maintenance
///
/// The methods in this set of operations do not interact with user I/O
pub trait MediaFilesMaintenanceOps {
    /// Counts the number of [`MediaFile`]s for the specified [`Media`]
    async fn count_media_files(&mut self, media_id: &MediaId) -> Result<i64>;
}

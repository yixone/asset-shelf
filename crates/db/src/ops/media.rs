use models::{entities::Media, types::MediaId};
use result::Result;

use crate::types::{DeleteResult, InsertResult};

/// Read operations for data associated with the [`Media`] domain
pub trait MediaReadOps {
    /// Returns [`Media`] for a given [`MediaId`]
    async fn get_media_by_id(&mut self, id: &MediaId) -> Result<Option<Media>> {
        self.get_media_by_ids(std::slice::from_ref(id))
            .await
            .map(|a| a.into_iter().next())
    }

    /// Returns a set of [`Media`] objects based on a set of IDs
    async fn get_media_by_ids(&mut self, ids: &[MediaId]) -> Result<Vec<Media>>;
}

/// Write operations for data associated with the [`Media`] domain
pub trait MediaWriteOps {
    /// Inserts a [`Media`] into the database
    async fn insert_media(&mut self, m: &Media) -> Result<InsertResult>;

    /// Removes [`Media`] from the database
    async fn delete_media(&mut self, id: &MediaId) -> Result<DeleteResult>;
}

/// Operations on data associated with the [`Media`] domain, used for maintenance
///
/// The methods in this set of operations do not interact with user I/O
pub trait MediaMaintenanceOps {
    /// Returns a list of [`Media`] that are not referenced by anyone
    async fn get_orphans_media(&mut self, limit: u32) -> Result<Vec<Media>>;
}

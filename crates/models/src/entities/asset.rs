use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    EntityError, Result,
    entities::FileGroup,
    id::{AssetId, FileGroupId},
    patches::{AssetFeaturesPatch, AssetMetaPatch, AssetPatch},
    ports::{AssetDatabase, file::FileGroupDatabase},
    types::{
        AssetSortBy, AssetState, Color, DeletedVisibility, Pagination, PerceptualHash, SortOrder,
    },
};

/// Represents an asset and its current state within the media library
///
/// An asset is a logical media item that may have multiple associated
/// file variants and derived features
#[derive(Debug)]
pub struct Asset {
    /// Unique asset identifier
    pub(crate) id: AssetId,

    /// Parrent asset id
    pub(crate) parent_id: Option<AssetId>,

    /// Identifier of the file group associated with the asset
    pub(crate) file_group_id: FileGroupId,

    /// Current lifecycle state of the asset
    pub(crate) state: AssetState,

    /// Asset features
    pub(crate) features: AssetFeatures,

    /// Asset metadata
    pub(crate) meta: AssetMeta,

    /// Identifier of the asset that this asset is considered a duplicate of
    pub(crate) duplicate_of: Option<AssetId>,

    /// Asset creation time
    pub(crate) created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub(crate) updated_at: Option<DateTime<Utc>>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub(crate) deleted_at: Option<DateTime<Utc>>,

    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub(crate) is_offline: bool,

    /// Type of the asset's original media file
    pub(crate) original_mime: MimeKind,
}

impl Asset {
    /// Returns the ID of this [`Asset`]
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Returns the ID of the parent [`Asset`]
    pub fn parent_id(&self) -> Option<AssetId> {
        self.parent_id
    }

    #[inline]
    /// Returns `true` if the [`Asset`] has a parent
    pub fn is_child(&self) -> bool {
        self.parent_id().is_some()
    }

    /// Returns the lifecycle state of this [`Asset`]
    pub fn state(&self) -> AssetState {
        self.state
    }

    /// Returns a reference to the derived features of this [`Asset`]
    pub fn features(&self) -> &AssetFeatures {
        &self.features
    }

    /// Returns a reference to the user-provided metadata of this [`Asset`]
    pub fn meta(&self) -> &AssetMeta {
        &self.meta
    }

    /// Returns the ID of the asset this [`Asset`] duplicates
    pub fn duplicate_of(&self) -> Option<AssetId> {
        self.duplicate_of
    }

    #[inline]
    /// Returns `true` if this [`Asset`] is marked as a duplicate
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of().is_some()
    }

    /// Returns the creation timestamp of this [`Asset`]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the last update timestamp of this [`Asset`]
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    /// Returns the deletion timestamp of this [`Asset`]
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[inline]
    /// Returns `true` if this [`Asset`] is marked as deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    /// Returns `true` if this [`Asset`] is marked as offline
    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    /// Returns the MIME kind of the original file associated with this [`Asset`]
    pub fn original_mime(&self) -> MimeKind {
        self.original_mime
    }

    /// Creates and persists a new [`Asset`] associated with the given [`FileGroup`]
    ///
    /// The new asset stats in the [`AssetState::Pending`] state
    ///
    /// Returns [`EntityError::InvalidRelation`] if the asset references itself
    /// as its parent or as the asset it duplicates
    ///
    /// Returns [`EntityError::NotFound`] if the file group has no original file
    pub async fn create<DB>(
        id: AssetId,
        parent: Option<&Asset>,
        data: AssetData,
        file_group: &FileGroup,
        db: &mut DB,
    ) -> Result<Self>
    where
        DB: AssetDatabase,
    {
        let parent_id = parent.map(|p| p.id());
        if let Some(parent_id) = parent_id
            && parent_id == id
        {
            // An asset cannot have itself as a parent
            return Err(EntityError::InvalidRelation);
        }

        if data.duplicate_of == Some(id) {
            // An asset cannot duplicate itself
            return Err(EntityError::InvalidRelation);
        }

        let original = file_group.original().ok_or(EntityError::NotFound)?;

        let meta = AssetMeta {
            name: data.name,
            caption: data.caption,
            source_url: data.source_url,
        };
        let features = AssetFeatures::default();

        let asset = Asset {
            id,
            parent_id,
            state: AssetState::Pending,
            created_at: Utc::now(),
            updated_at: None,
            deleted_at: None,
            features,
            file_group_id: file_group.id.clone(),
            duplicate_of: data.duplicate_of,
            meta,
            is_offline: false,
            original_mime: original.mime_kind(),
        };

        db.insert_asset(&asset).await?;

        Ok(asset)
    }

    /// Loads an [`Asset`] and its associated [`FileGroup`] by ID
    ///
    /// Returns [`EntityError::NotFound`] if the asset does not exist
    pub async fn load<DB>(id: AssetId, db: &mut DB) -> Result<(Self, FileGroup)>
    where
        DB: AssetDatabase,
    {
        db.get_asset(id).await?.ok_or(EntityError::NotFound)
    }

    /// Loads the [`FileGroup`] associated with this [`Asset`]
    ///
    /// Returns [`EntityError::NotFound`] if the file group does not exist
    pub async fn load_file_group<DB>(&self, db: &mut DB) -> Result<FileGroup>
    where
        DB: FileGroupDatabase,
    {
        FileGroup::load(&self.file_group_id, db).await
    }

    /// Lists assets and their associated file groups using the specified
    /// pagination, ordering and deletion visibility
    pub async fn list<DB>(
        pagination: Pagination,
        order: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
        db: &mut DB,
    ) -> Result<Vec<(Asset, FileGroup)>>
    where
        DB: AssetDatabase,
    {
        db.list_assets(pagination, order, deleted).await
    }

    /// Updates the [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update<DB>(&mut self, patch: AssetPatch, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let updated = db.update_asset(self.id, &patch, Utc::now()).await?;
        *self = updated;

        Ok(())
    }

    /// Updates the lifecycle state of this [`Asset`]
    pub async fn update_state<DB>(&mut self, state: AssetState, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        self.update(AssetPatch::new().state(state), db).await
    }

    /// Marks this [`Asset`] as deleted
    pub async fn mark_deleted<DB>(&mut self, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        self.update(AssetPatch::new().deleted_at(Some(Utc::now())), db)
            .await
    }

    /// Restores this [`Asset`] from the deleted state
    pub async fn restore_deleted<DB>(&mut self, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        self.update(AssetPatch::new().deleted_at(None), db).await
    }

    /// Updates the user-provided metadata of this [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update_meta<DB>(&mut self, patch: AssetMetaPatch, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let now = Utc::now();

        let updated = db.update_asset_meta(self.id, &patch, now).await?;

        self.meta = updated;
        self.updated_at = Some(now);

        Ok(())
    }

    /// Updates the derived features of this [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update_features<DB>(
        &mut self,
        patch: AssetFeaturesPatch,
        db: &mut DB,
    ) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let now = Utc::now();

        let updated = db.update_asset_features(self.id, &patch, now).await?;

        self.features = updated;
        self.updated_at = Some(now);

        Ok(())
    }

    /// Marks this [`Asset`] as offline
    pub async fn set_offline<DB>(&mut self, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        self.update(AssetPatch::new().is_offline(true), db).await
    }

    /// Unmarks this [`Asset`] as offline
    pub async fn unset_offline<DB>(&mut self, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        self.update(AssetPatch::new().is_offline(false), db).await
    }

    /// Deletes this [`Asset`] from persistent storage
    ///
    /// Consumes the instance
    pub async fn delete<DB>(self, db: &mut DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        if !db.delete_asset(self.id).await? {
            return Err(EntityError::NotFound);
        }
        Ok(())
    }
}

/// Persisted representation of an [`Asset`] used to reconstruct
/// the domain entity
///
/// This type is intended for use by persistence implementations
/// and is not part of the regular domain API
#[cfg(feature = "dev")]
pub struct AssetPersistence {
    pub id: AssetId,
    pub parent_id: Option<AssetId>,
    pub file_group_id: FileGroupId,
    pub state: AssetState,
    pub accent_color: Option<Color>,
    pub p_hash: Option<PerceptualHash>,
    pub a_hash: Option<PerceptualHash>,
    pub dimension: Option<(u32, u32)>,
    pub meta: AssetData,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_offline: bool,
    pub original_mime: MimeKind,
}

#[cfg(feature = "dev")]
impl Asset {
    /// Creates [`Asset`] from persisted data
    ///
    /// This method is intended for use by persistence implementations
    /// when loading an asset from storage
    pub fn from_persistence(p: AssetPersistence) -> Self {
        Asset {
            id: p.id,
            parent_id: p.parent_id,
            file_group_id: p.file_group_id,
            state: p.state,
            features: AssetFeatures {
                accent_color: p.accent_color,
                p_hash: p.p_hash,
                a_hash: p.a_hash,
                dimension: p.dimension,
            },
            meta: AssetMeta {
                name: p.meta.name,
                caption: p.meta.caption,
                source_url: p.meta.source_url,
            },
            duplicate_of: p.meta.duplicate_of,
            created_at: p.created_at,
            updated_at: p.updated_at,
            deleted_at: p.deleted_at,
            is_offline: p.is_offline,
            original_mime: p.original_mime,
        }
    }
}

/// Derived features calculated from an asset's media
///
/// Features are generated from the asset's media and may be unavailable
/// while media processing has not completed
#[derive(Debug)]
pub struct AssetFeatures {
    /// Asset accent color
    pub(crate) accent_color: Option<Color>,

    /// Asset perceptual hash
    ///
    /// Used to compare the visual similarity of media
    pub(crate) p_hash: Option<PerceptualHash>,

    /// Average asset hash
    ///
    /// Used to compare the visual similarity of media
    pub(crate) a_hash: Option<PerceptualHash>,

    /// Asset width and height in pixels
    pub(crate) dimension: Option<(u32, u32)>,
}

impl AssetFeatures {
    /// Returns the asset width in pixels
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, _)| w)
    }

    /// Returns the asset height in pixels
    pub fn height(&self) -> Option<u32> {
        self.dimension.map(|(_, h)| h)
    }

    /// Returns the aspect ratio of the asset
    ///
    /// Returns `None` if the asset dimensions are unavailable
    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.dimension {
            Some(w as f32 / h as f32)
        } else {
            None
        }
    }

    /// Returns `true` if all required derived features have been calculated
    pub fn enough_fields(&self) -> bool {
        self.accent_color.is_some()
            && self.p_hash.is_some()
            && self.a_hash.is_some()
            && self.dimension.is_some()
    }

    /// Returns the pHash of this [`AssetFeatures`]
    pub fn p_hash(&self) -> Option<PerceptualHash> {
        self.p_hash
    }

    /// Returns the aHash of this [`AssetFeatures`]
    pub fn a_hash(&self) -> Option<PerceptualHash> {
        self.a_hash
    }

    /// Returns the accent color of this [`AssetFeatures`]
    pub fn accent_color(&self) -> Option<Color> {
        self.accent_color
    }
}

/// User-provided metadata associated with an asset
#[derive(Debug)]
pub struct AssetMeta {
    /// Optional name for the asset
    pub(crate) name: Option<String>,

    /// Optional caption for the asset
    pub(crate) caption: Option<String>,

    /// URL from which the asset was obtained
    pub(crate) source_url: Option<String>,
}

impl AssetMeta {
    /// Returns the name of this [`AssetMeta`]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the caption of this [`AssetMeta`]
    pub fn caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    /// Returns the source url of this [`AssetMeta`]
    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }
}

/// A lightweight representation of an asset containing only its core state
#[derive(Debug)]
pub struct AssetDehydrated {
    /// Unique asset identifier
    pub id: AssetId,

    /// Current lifecycle state of the asset
    pub state: AssetState,

    /// Asset creation time
    pub created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub updated_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Data required to create an [`Asset`]
pub struct AssetData {
    /// Identifier of the asset that this asset is considered a duplicate of
    pub duplicate_of: Option<AssetId>,

    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,

    /// URL from which the asset was obtained
    pub source_url: Option<String>,
}

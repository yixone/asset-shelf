use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    EntityError,
    entities::FileGroup,
    id::{AssetId, FileGroupId},
    patches::{AssetFeaturesPatch, AssetMetaPatch, AssetPatch},
    ports::{AssetDatabase, AssetWriter, FileGroupDatabase},
    result::Result,
    types::{AssetState, Color, PerceptualHash},
};

/// Represents an asset and its current state within the media library
///
/// An asset is a logical media item that may have multiple associated
/// file variants and derived features
#[derive(Debug)]
pub struct Asset {
    /// Unique asset identifier
    pub(crate) id: AssetId,

    /// Current lifecycle state of the asset
    pub(crate) state: AssetState,

    /// Asset creation time
    pub(crate) created_at: DateTime<Utc>,

    /// Time of the asset's last modification
    pub(crate) updated_at: DateTime<Utc>,

    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub(crate) deleted_at: Option<DateTime<Utc>>,

    /// Asset features
    pub(crate) features: AssetFeatures,

    /// Identifier of the file group associated with the asset
    pub(crate) file_group_id: FileGroupId,

    /// Identifier of the asset that this asset is considered a duplicate of
    pub(crate) duplicate_of: Option<AssetId>,

    /// Asset metadata
    pub(crate) meta: AssetMeta,

    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub(crate) is_offline: bool,

    /// Type of the asset's original media file
    pub(crate) original_mime: MimeKind,
}

impl Asset {
    /// Creates and persists a new [`Asset`] associated with the given [`FileGroup`]
    ///
    /// Returns [`EntityError::NotFound`] if the file group does not contain
    /// an original file
    pub async fn create<DB>(
        id: AssetId,
        data: AssetData,
        file_group: &FileGroup,
        db: &mut DB,
    ) -> Result<Self>
    where
        DB: AssetWriter,
    {
        let meta = AssetMeta {
            name: data.name,
            caption: data.caption,
            source_url: data.source_url,
        };

        let features = AssetFeatures {
            accent_color: None,
            p_hash: None,
            a_hash: None,
            dimension: None,
        };

        let original = file_group.original().ok_or(EntityError::NotFound)?;
        let now = Utc::now();

        let asset = Asset {
            id,
            state: AssetState::Pending,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            features,
            file_group_id: file_group.id().clone(),
            duplicate_of: data.duplicate_of,
            meta,
            is_offline: false,
            original_mime: original.mime_kind(),
        };

        db.insert_asset(&asset).await?;

        Ok(asset)
    }

    /// Loads an [`Asset`] together with its associated [`FileGroup`]
    ///
    /// Returns [`EntityError::NotFound`] if the asset does not exist
    pub async fn get<DB>(id: AssetId, db: &DB) -> Result<(Asset, FileGroup)>
    where
        DB: AssetDatabase,
    {
        db.get_asset(id).await?.ok_or(EntityError::NotFound)
    }

    /// Updates the [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update<DB>(&mut self, patch: AssetPatch, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        db.update_asset(self.id(), &patch).await?;
        patch.apply_domain(self);
        Ok(())
    }

    /// Updates the lifecycle state of this [`Asset`]
    pub async fn update_state<DB>(&mut self, state: AssetState, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let patch = AssetPatch::new().state(state);
        self.update(patch, db).await
    }

    /// Marks this [`Asset`] as deleted
    pub async fn mark_deleted<DB>(&mut self, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let patch = AssetPatch::new().deleted_at(Some(Utc::now()));
        self.update(patch, db).await
    }

    /// Restores this [`Asset`] from the deleted state
    pub async fn restore_deleted<DB>(&mut self, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        let patch = AssetPatch::new().deleted_at(None);
        self.update(patch, db).await
    }

    /// Loads the [`FileGroup`] associated with this [`Asset`]
    ///
    /// Returns [`EntityError::NotFound`] if the file group does not exist
    pub async fn file_group<DB>(&self, db: &DB) -> Result<FileGroup>
    where
        DB: FileGroupDatabase,
    {
        FileGroup::load(self.file_group_id(), db).await
    }

    /// Returns the id of this [`Asset`]
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Returns the state of this [`Asset`]
    pub fn state(&self) -> AssetState {
        self.state
    }

    /// Returns this [`Asset`] creation time
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns this [`Asset`] update time
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns this [`Asset`] deletion time
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    /// Returns `true` if the asset has been marked as deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Returns `true` if the asset is not marked as deleted
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// Returns `true` if the asset is offline
    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    /// Returns `true` if the asset is marked as a duplicate of another asset
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of.is_some()
    }

    /// Returns `true` if the asset has all data
    /// required for use
    pub fn is_available(&self) -> bool {
        !self.is_deleted() && !self.is_offline
    }

    /// Returns a reference to the features of this [`Asset`]
    pub fn features(&self) -> &AssetFeatures {
        &self.features
    }

    /// Updates the derived features of this [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update_features<DB>(&mut self, patch: AssetFeaturesPatch, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        db.update_asset_features(self.id(), &patch).await?;
        patch.apply_domain(&mut self.features);
        Ok(())
    }

    /// Returns a reference to the file group id of this [`Asset`]
    pub fn file_group_id(&self) -> &FileGroupId {
        &self.file_group_id
    }

    /// Returns a reference to the meta of this [`Asset`]
    pub fn meta(&self) -> &AssetMeta {
        &self.meta
    }

    /// Updates the user-provided metadata of this [`Asset`] using the specified patch
    ///
    /// The domain model is updated only after the persistence operation succeeds
    pub async fn update_meta<DB>(&mut self, patch: AssetMetaPatch, db: &DB) -> Result<()>
    where
        DB: AssetDatabase,
    {
        db.update_asset_meta(self.id(), &patch).await?;
        patch.apply_domain(&mut self.meta);
        Ok(())
    }

    /// Returns the MIME kind of this [`Asset`]'s original media file
    pub fn original_mime(&self) -> MimeKind {
        self.original_mime
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

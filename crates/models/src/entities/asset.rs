use chrono::{DateTime, Utc};
use mime::MimeKind;

use crate::{
    EntityError, Result,
    entities::FileGroup,
    id::{AssetId, FileGroupId},
    patches::{AssetFeaturesPatch, AssetMetaPatch, AssetPatch, PatchField},
    types::{AssetState, Color, PerceptualHash},
};

/// Represents an asset and its current state within the media library
///
/// An asset is a logical media item that may have multiple associated
/// file variants and derived features
#[derive(Debug)]
pub struct Asset {
    /// Unique asset ID
    pub(crate) id: AssetId,

    /// Asset features
    pub(crate) features: AssetFeatures,
    /// User-provided asset metadata
    pub(crate) meta: AssetMeta,

    /// Parrent asset ID
    pub(crate) parent_id: Option<AssetId>,
    /// ID of the asset that this asset is considered a duplicate of
    pub(crate) duplicate_of: Option<AssetId>,

    /// Identifier of the file group associated with the asset
    pub(crate) file_group_id: FileGroupId,
    //// Original asset file name
    pub(crate) original_filename: Option<String>,
    /// Type of the asset's original media file
    pub(crate) original_mime: MimeKind,

    /// Current lifecycle state of the asset
    pub(crate) state: AssetState,
    /// If `true`, the original file has been lost from storage and the asset cannot be used
    pub(crate) is_offline: bool,

    /// Asset creation time
    pub(crate) created_at: DateTime<Utc>,
    /// Time of the asset's last modification
    pub(crate) updated_at: Option<DateTime<Utc>>,
    /// Optional asset deletion time
    ///
    /// If `Some`, the asset is considered deleted
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}

/// Derived features calculated from an asset's media
///
/// Features are generated from the asset's media and may be unavailable
/// while media processing has not completed
#[derive(Debug, Default)]
pub struct AssetFeatures {
    /// Asset accent color
    pub(crate) accent_color: Option<Color>,

    /// Asset width and height in pixels
    pub(crate) dimension: Option<(u32, u32)>,

    /// Asset perceptual hash
    pub(crate) p_hash: Option<PerceptualHash>,
    /// Average asset hash
    pub(crate) a_hash: Option<PerceptualHash>,
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

/// Data required to create an [`Asset`]
pub struct AssetData {
    /// Optional name for the asset
    pub name: Option<String>,

    /// Optional caption for the asset
    pub caption: Option<String>,

    /// URL from which the asset was obtained
    pub source_url: Option<String>,
}

impl Asset {
    /// Creates a new [`Asset`]
    pub fn new(
        id: AssetId,
        parent_id: Option<AssetId>,
        duplicate_of: Option<AssetId>,
        original_filename: Option<String>,
        data: AssetData,
        files: &FileGroup,
    ) -> Result<Self> {
        if parent_id == Some(id) {
            return Err(EntityError::InvalidRelation);
        }
        if duplicate_of == Some(id) {
            return Err(EntityError::InvalidRelation);
        }

        let Some(original) = files.original() else {
            return Err(EntityError::NotFound);
        };

        let features = AssetFeatures::default();
        let meta = AssetMeta {
            name: data.name,
            caption: data.caption,
            source_url: data.source_url,
        };

        let asset = Asset {
            id,
            features,
            meta,
            parent_id,
            duplicate_of,
            file_group_id: files.id().clone(),
            original_filename,
            original_mime: original.mime_kind(),
            state: AssetState::Pending,
            is_offline: false,
            created_at: Utc::now(),
            updated_at: None,
            deleted_at: None,
        };

        Ok(asset)
    }

    /// Applies a patch to the asset
    ///
    /// Rejects changes that make asset its own parent or duplicate
    pub fn apply_patch(&mut self, patch: AssetPatch) -> Result<()> {
        if let PatchField::Set(Some(p)) = patch.parent_id
            && p == self.id
        {
            return Err(EntityError::InvalidRelation);
        }

        if let PatchField::Set(Some(d)) = patch.duplicate_of
            && d == self.id
        {
            return Err(EntityError::InvalidRelation);
        }

        patch.apply_domain(self);
        Ok(())
    }

    /// Applies a patch without validating domain invariants
    ///
    /// The caller is responsible for ensuring that the resulting asset
    /// satisfies all domain invariants
    pub fn apply_patch_unchecked(&mut self, patch: AssetPatch) {
        patch.apply_domain(self);
    }

    /// Returns the ID of this [`Asset`]
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Returns a reference to the derived features of this [`Asset`]
    pub fn features(&self) -> &AssetFeatures {
        &self.features
    }

    /// Returns a mutable reference to the derived features of this [`Asset`]
    pub fn features_mut(&mut self) -> &mut AssetFeatures {
        &mut self.features
    }

    /// Returns a reference to the user-provided metadata of this [`Asset`]
    pub fn meta(&self) -> &AssetMeta {
        &self.meta
    }

    /// Returns a mutable reference to the user-provided metadata of this [`Asset`]
    pub fn meta_mut(&mut self) -> &mut AssetMeta {
        &mut self.meta
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

    /// Returns the ID of the asset this [`Asset`] duplicates
    pub fn duplicate_of(&self) -> Option<AssetId> {
        self.duplicate_of
    }

    #[inline]
    /// Returns `true` if this [`Asset`] is marked as a duplicate
    pub fn is_duplicate(&self) -> bool {
        self.duplicate_of().is_some()
    }

    /// Returns the ID of the group of files associated with this asset
    pub fn file_group_id(&self) -> &FileGroupId {
        &self.file_group_id
    }

    /// Returns the name of the original file associated with this [`Asset`]
    pub fn original_filename(&self) -> Option<&str> {
        self.original_filename.as_deref()
    }

    /// Returns the MIME kind of the original file associated with this [`Asset`]
    pub fn original_mime(&self) -> MimeKind {
        self.original_mime
    }

    /// Returns the lifecycle state of this [`Asset`]
    pub fn state(&self) -> AssetState {
        self.state
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
    pub duplicate_of: Option<AssetId>,
    pub file_group_id: FileGroupId,
    pub state: AssetState,
    pub accent_color: Option<Color>,
    pub p_hash: Option<PerceptualHash>,
    pub a_hash: Option<PerceptualHash>,
    pub dimension: Option<(u32, u32)>,
    pub data: AssetData,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_offline: bool,
    pub original_mime: MimeKind,
    pub original_filename: Option<String>,
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
            duplicate_of: p.duplicate_of,
            file_group_id: p.file_group_id,
            state: p.state,
            features: AssetFeatures {
                accent_color: p.accent_color,
                p_hash: p.p_hash,
                a_hash: p.a_hash,
                dimension: p.dimension,
            },
            meta: AssetMeta {
                name: p.data.name,
                caption: p.data.caption,
                source_url: p.data.source_url,
            },
            created_at: p.created_at,
            updated_at: p.updated_at,
            deleted_at: p.deleted_at,
            is_offline: p.is_offline,
            original_mime: p.original_mime,
            original_filename: p.original_filename,
        }
    }
}

impl AssetFeatures {
    /// Applies a patch to the asset features
    pub fn apply_patch(&mut self, patch: AssetFeaturesPatch) {
        patch.apply_domain(self);
    }

    /// Returns the accent color of this [`AssetFeatures`]
    pub fn accent_color(&self) -> Option<Color> {
        self.accent_color
    }

    /// Returns the aspect ratio of the asset
    ///
    /// Returns `None` if the asset dimensions are unavailable
    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.dimension {
            // Prevents division by `0`
            if h == 0 {
                return None;
            }

            Some(w as f32 / h as f32)
        } else {
            None
        }
    }

    /// Returns the asset width in pixels
    pub fn width(&self) -> Option<u32> {
        self.dimension.map(|(w, ..)| w)
    }

    /// Returns the asset height in pixels
    pub fn height(&self) -> Option<u32> {
        self.dimension.map(|(_, h)| h)
    }

    /// Returns the pHash of this [`AssetFeatures`]
    pub fn p_hash(&self) -> Option<PerceptualHash> {
        self.p_hash
    }

    /// Returns the aHash of this [`AssetFeatures`]
    pub fn a_hash(&self) -> Option<PerceptualHash> {
        self.a_hash
    }
}

impl AssetMeta {
    /// Applies a patch to the asset features
    pub fn apply_patch(&mut self, patch: AssetMetaPatch) {
        patch.apply_domain(self);
    }

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

use chrono::{DateTime, Utc};
use db_api::DatabaseError;
use mime::MimeKind;
use models::{
    entities::{
        Asset,
        asset::{AssetData, AssetPersistence, AssetRelData},
    },
    id::{AssetId, FileGroupId},
    types::{AssetFailure, AssetState, Color, PerceptualHash, Sha256Checksum},
};
use sqlx::prelude::FromRow;

/// Persisted [`Asset`] data stored in the `assets` table
#[derive(FromRow)]
pub struct DbAsset {
    pub id: AssetId,

    pub parent_id: Option<AssetId>,
    pub duplicate_of: Option<AssetId>,

    pub file_group_id: FileGroupId,
    pub checksum: Vec<u8>,
    pub original_filename: Option<String>,
    pub original_mime_kind: MimeKind,

    pub state: String,
    pub state_failure: Option<AssetFailure>,
    pub is_offline: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Persisted features of the [`Asset`] stored in the `assets_features` table
#[derive(FromRow)]
pub struct DbAssetFeatures {
    pub id: AssetId,

    pub accent_color: Option<i32>,

    pub width: Option<i32>,
    pub height: Option<i32>,

    pub p_hash: Option<i64>,
    pub a_hash: Option<i64>,
}

/// Persisted user-provided metadata of the [`Asset`] stored in the `asset_meta` table
#[derive(FromRow)]
pub struct DbAssetMeta {
    pub id: AssetId,

    pub name: Option<String>,
    pub caption: Option<String>,
    pub source_url: Option<String>,
}

impl DbAsset {
    /// Converts a domain asset into records for the SQLite schema
    pub fn from_domain(a: &Asset) -> (DbAsset, DbAssetFeatures, DbAssetMeta) {
        let features = a.features();
        let db_features = DbAssetFeatures {
            id: a.id(),
            accent_color: features.accent_color().map(|c| c.as_int()),
            width: features.width().map(|w| w as i32),
            height: features.height().map(|w| w as i32),
            p_hash: features.p_hash().map(|p| p.as_int()),
            a_hash: features.a_hash().map(|a| a.as_int()),
        };

        let meta = a.meta();
        let db_meta = DbAssetMeta {
            id: a.id(),
            name: meta.name().map(str::to_owned),
            caption: meta.caption().map(str::to_owned),
            source_url: meta.source_url().map(str::to_owned),
        };

        let db_asset = DbAsset {
            id: a.id(),
            parent_id: a.parent_id(),
            duplicate_of: a.duplicate_of(),
            file_group_id: a.file_group_id().clone(),
            checksum: a.checksum().to_vec(),
            original_filename: a.original_filename().map(str::to_owned),
            original_mime_kind: a.original_mime(),
            state: a.state().to_string(),
            state_failure: match a.state() {
                AssetState::Failed(asset_failure) => Some(asset_failure),
                _ => None,
            },
            is_offline: a.is_offline(),
            created_at: a.created_at(),
            updated_at: a.updated_at(),
            deleted_at: a.deleted_at(),
        };

        (db_asset, db_features, db_meta)
    }

    /// Reconstructs a domain asset from its persisted records
    pub fn into_domain(
        (a, f, m): (DbAsset, DbAssetFeatures, DbAssetMeta),
    ) -> Result<Asset, DatabaseError> {
        let d = Asset::from_persistence(AssetPersistence {
            id: a.id,
            rels: AssetRelData {
                parent_id: a.parent_id,
                duplicate_of: a.duplicate_of,
            },
            data: AssetData {
                name: m.name,
                caption: m.caption,
                source_url: m.source_url,
            },
            file_group_id: a.file_group_id,
            state: AssetState::from_persistence(&a.state, a.state_failure)
                .ok_or(DatabaseError::InvalidStoredData)?,
            accent_color: f.accent_color.map(Color::from),
            p_hash: f.p_hash.map(PerceptualHash::from),
            a_hash: f.a_hash.map(PerceptualHash::from),
            dimension: match (f.width, f.height) {
                (Some(w), Some(h)) => Some((w as u32, h as u32)),
                _ => None,
            },
            created_at: a.created_at,
            updated_at: a.updated_at,
            deleted_at: a.deleted_at,
            is_offline: a.is_offline,
            original_mime: a.original_mime_kind,
            checksum: Sha256Checksum::new(
                a.checksum
                    .try_into()
                    .map_err(|_| DatabaseError::InvalidStoredData)?,
            ),
            original_filename: a.original_filename,
        });

        Ok(d)
    }
}

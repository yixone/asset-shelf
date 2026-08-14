use chrono::{DateTime, Utc};
use models::{
    collections::{
        Collection, CollectionAdditions, CollectionAsset,
        view::{CollectionItemView, CollectionView},
    },
    media::MediaVariant,
};
use serde::Serialize;

use crate::{dto::v1::assets::AssetDtoV1, utils::url::build_media_url};

#[derive(Debug, Serialize)]
pub struct CollectionDtoV1 {
    id: String,
    name: String,
    description: Option<String>,
    assets_count: u32,
    thumbnails: Vec<String>,
    created_at: DateTime<Utc>,
}

impl From<CollectionView> for CollectionDtoV1 {
    fn from(q: CollectionView) -> Self {
        let collection = q.inner;
        let additions = q.add;
        CollectionDtoV1::from((collection, additions))
    }
}

impl From<(Collection, CollectionAdditions)> for CollectionDtoV1 {
    fn from((c, ca): (Collection, CollectionAdditions)) -> Self {
        CollectionDtoV1 {
            id: c.id.to_string(),
            name: c.name,
            description: c.description,
            assets_count: ca.assets_count,
            thumbnails: ca
                .thumbnails
                .into_iter()
                .map(|t| build_media_url(&t, MediaVariant::Thumbnail))
                .collect(),
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CollectionAssetDtoV1 {
    relation: String,
    added_at: DateTime<Utc>,
    asset: AssetDtoV1,
}

impl From<CollectionItemView> for CollectionAssetDtoV1 {
    fn from(q: CollectionItemView) -> Self {
        let ca = q.inner;
        let asset = q.asset;
        CollectionAssetDtoV1::from((ca, asset))
    }
}

impl<T> From<(CollectionAsset, T)> for CollectionAssetDtoV1
where
    T: Into<AssetDtoV1>,
{
    fn from((ca, a): (CollectionAsset, T)) -> Self {
        CollectionAssetDtoV1 {
            relation: ca.id.to_string(),
            added_at: ca.added_at,
            asset: a.into(),
        }
    }
}

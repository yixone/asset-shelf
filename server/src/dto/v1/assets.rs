use chrono::{DateTime, Utc};
use db::queries::asset::AssetQuery;
use mimetype::MimeKind;
use models::entities::{Asset, AssetFeatures, AssetState};
use serde::Serialize;

use crate::dto::v1::media::MediaGroupDtoV1;

#[derive(Debug, Serialize)]
pub struct AssetDtoV1 {
    id: String,
    state: AssetState,

    #[serde(flatten)]
    media: MediaGroupDtoV1,

    #[serde(rename = "type")]
    media_type: MimeKind,

    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,

    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,

    width: Option<u32>,
    height: Option<u32>,

    color: Option<String>,
}

impl From<AssetQuery> for AssetDtoV1 {
    fn from(q: AssetQuery) -> Self {
        let asset = q.inner;
        let asset_features = q.features;
        let media = q.media;
        AssetDtoV1::from((asset, asset_features, media))
    }
}

impl<M> From<(Asset, AssetFeatures, M)> for AssetDtoV1
where
    M: Into<MediaGroupDtoV1>,
{
    fn from((asset, asset_features, media): (Asset, AssetFeatures, M)) -> Self {
        AssetDtoV1 {
            id: asset.id.to_string(),
            state: asset.state,
            media: media.into(),
            media_type: asset.media_type,
            created_at: asset.created_at,
            deleted_at: asset.deleted_at,
            title: asset.title,
            caption: asset.caption,
            source_url: asset.source_url,
            width: asset_features.width,
            height: asset_features.height,
            color: asset_features.accent_color.map(|c| c.hex()),
        }
    }
}

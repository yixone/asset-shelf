use chrono::{DateTime, Utc};
use models::{
    entities::{Asset, AssetState},
    types::AssetId,
};
use serde::Serialize;

use crate::dto::v1::media::MediaGroupDtoV1;

#[derive(Debug, Serialize)]
pub struct AssetDtoV1 {
    id: AssetId,
    state: AssetState,

    media: MediaGroupDtoV1,

    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,

    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,
}

impl<M> From<(Asset, M)> for AssetDtoV1
where
    M: Into<MediaGroupDtoV1>,
{
    fn from((asset, media): (Asset, M)) -> Self {
        AssetDtoV1 {
            id: asset.id,
            state: asset.state,
            media: media.into(),
            created_at: asset.created_at,
            deleted_at: asset.deleted_at,
            title: asset.title,
            caption: asset.caption,
            source_url: asset.source_url,
        }
    }
}

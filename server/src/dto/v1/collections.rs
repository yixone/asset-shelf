use chrono::{DateTime, Utc};
use models::entities::{Collection, CollectionAdditions, MediaVariant};
use serde::Serialize;

use crate::utils::url::build_media_url;

#[derive(Debug, Serialize)]
pub struct CollectionDtoV1 {
    id: String,
    name: String,
    description: Option<String>,
    assets_count: u32,
    thumbnails: Vec<String>,
    created_at: DateTime<Utc>,
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

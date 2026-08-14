use flake_id::str::FlakeIdStr;
use models::{
    collections::CollectionAdditions,
    types::{CollectionId, MediaId},
};
use sqlx::FromRow;

/// Raw database row for mapping in [`CollectionAdditions`]
#[derive(FromRow)]
pub struct CollectionAdditionsRow {
    pub id: CollectionId,
    /// The list of previews is fetched as a concatenated list of
    /// MediaIds and split back apart upon entering the domain
    pub thumbnails: String,
    pub assets_count: u32,
}

impl From<CollectionAdditionsRow> for CollectionAdditions {
    fn from(row: CollectionAdditionsRow) -> Self {
        let ids = row
            .thumbnails
            .split(';')
            .filter(|m| !m.trim().is_empty())
            .map(|v| MediaId(FlakeIdStr(v.to_string())))
            .collect::<Vec<_>>();

        CollectionAdditions {
            collection: row.id,
            thumbnails: ids,
            assets_count: row.assets_count,
        }
    }
}

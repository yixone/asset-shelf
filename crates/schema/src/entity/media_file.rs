use crate::{
    id::{MediaFileId, MediaId},
    types::MediaVariant,
};

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct MediaFile {
    pub id: MediaFileId,
    pub media_id: MediaId,

    pub variant: MediaVariant,
    pub storage_path: String,
}

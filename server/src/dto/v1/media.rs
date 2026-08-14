use std::collections::BTreeMap;

use mimetype::MimeType;
use models::media::{MediaFile, MediaVariant};
use serde::Serialize;

use crate::utils::url::build_media_url;

#[derive(Debug, Serialize)]
pub struct MediaGroupDtoV1 {
    media: BTreeMap<MediaVariant, MediaFileDtoV1>,
}

#[derive(Debug, Serialize)]
pub struct MediaFileDtoV1 {
    url: String,
    size_bytes: i64,
    mimetype: MimeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_milis: Option<i64>,
}

impl From<Vec<MediaFile>> for MediaGroupDtoV1 {
    fn from(files: Vec<MediaFile>) -> Self {
        let files = files
            .into_iter()
            .map(|f| (f.variant, f.into()))
            .collect::<BTreeMap<_, _>>();

        MediaGroupDtoV1 { media: files }
    }
}

impl From<MediaFile> for MediaFileDtoV1 {
    fn from(file: MediaFile) -> Self {
        MediaFileDtoV1 {
            url: build_media_url(&file.media_id, file.variant),
            size_bytes: file.size_bytes,
            mimetype: file.mimetype,
            duration_milis: file.duration_ms,
        }
    }
}

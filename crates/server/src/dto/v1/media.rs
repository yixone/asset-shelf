use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use mimetype::MimeType;
use models::entities::{Media, MediaFile, MediaVariant};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MediaGroupDtoV1 {
    files: BTreeMap<MediaVariant, MediaFileDtoV1>,
    group_created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MediaFileDtoV1 {
    url: String,
    size_bytes: i64,
    mimetype: MimeType,
}

impl From<(Media, Vec<MediaFile>)> for MediaGroupDtoV1 {
    fn from((group, files): (Media, Vec<MediaFile>)) -> Self {
        let files = files
            .into_iter()
            .map(|f| (f.variant, f.into()))
            .collect::<BTreeMap<_, _>>();

        MediaGroupDtoV1 {
            files,
            group_created_at: group.created_at,
        }
    }
}

impl From<MediaFile> for MediaFileDtoV1 {
    fn from(file: MediaFile) -> Self {
        MediaFileDtoV1 {
            url: build_media_file_url(&file),
            size_bytes: file.size_bytes,
            mimetype: file.mimetype,
        }
    }
}

pub fn build_media_file_url(f: &MediaFile) -> String {
    format!("/v1/media/{}?format={}", f.media_id, f.variant)
}

use models::{media::MediaVariant, types::MediaId};

pub fn build_media_url(media_id: &MediaId, variant: MediaVariant) -> String {
    format!("/v1/media/{}/{}", media_id, variant)
}

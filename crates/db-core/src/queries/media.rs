use models::entities::{Media, MediaFile};

pub struct MediaQuery {
    pub inner: Media,
    pub fiels: Vec<MediaFile>,
}

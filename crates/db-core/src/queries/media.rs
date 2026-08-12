use join::JoinBuilder;
use models::entities::{Media, MediaFile};

#[derive(Debug, Clone)]
pub struct MediaQuery {
    pub inner: Media,
    pub files: Vec<MediaFile>,
}

impl MediaQuery {
    pub fn from_domains(media: Vec<Media>, files: Vec<MediaFile>) -> Vec<MediaQuery> {
        JoinBuilder::new(media)
            .with_group(files, |m| m)
            .build_as(|(m, f)| MediaQuery { inner: m, files: f })
    }
}

use join::JoinBuilder;

use crate::media::{Media, MediaFile};

#[derive(Debug, Clone)]
pub struct MediaView {
    pub inner: Media,
    pub files: Vec<MediaFile>,
}

impl MediaView {
    /// Assembles the [`MediaView`] from models
    pub fn from_models(media: Vec<Media>, files: Vec<MediaFile>) -> Vec<MediaView> {
        JoinBuilder::new(media)
            .with_group(files, |m| m)
            .build_as(MediaView::from)
    }
}

impl From<(Media, Vec<MediaFile>)> for MediaView {
    fn from((m, f): (Media, Vec<MediaFile>)) -> Self {
        MediaView { inner: m, files: f }
    }
}

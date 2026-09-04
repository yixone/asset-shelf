use std::collections::HashSet;

use crate::{
    join::JoinBuilder,
    media::{Media, MediaFile, MediaVariant},
};

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

    /// Returns the variants that have already been generated for the given [`MediaView`]
    pub fn media_variants(&self) -> HashSet<MediaVariant> {
        let mut variants = HashSet::with_capacity(self.files.len());
        for v in &self.files {
            variants.insert(v.variant);
        }
        variants
    }
}

impl From<(Media, Vec<MediaFile>)> for MediaView {
    fn from((m, f): (Media, Vec<MediaFile>)) -> Self {
        MediaView { inner: m, files: f }
    }
}

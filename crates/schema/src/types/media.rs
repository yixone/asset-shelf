//! Shared types related to media files and the domain

auto_derived_ty! {
    /// Generalized media type
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum MediaType {
        /// Image-type media
        Image,
        /// Video-type media
        Video,
    }
}

impl MediaType {
    /// Returns `true` if the current [`MediaType`] is an image
    pub fn is_image(&self) -> bool {
        matches!(self, MediaType::Image)
    }

    /// Returns `true` if the current [`MediaType`] is a video
    pub fn is_video(&self) -> bool {
        matches!(self, MediaType::Video)
    }
}

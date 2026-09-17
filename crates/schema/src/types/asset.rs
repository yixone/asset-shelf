auto_derived_ty! {
    /// Represents the current lifecycle state of an asset
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum AssetState {
        /// The asset has been uploaded and is awaiting processing
        Pending,

        /// The asset is being processed
        Processing,

        /// The asset has been processed and is ready for use
        Ready,

        /// Asset processing failed with an error
        Failed,
    }
}

auto_derived_ty! {
    /// Generalized asset type
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum AssetType {
        /// Image-type media
        Image,
        /// Video-type media
        Video,
    }
}

impl AssetType {
    /// Returns `true` if the current [`AssetType`] is an image
    pub fn is_image(&self) -> bool {
        matches!(self, AssetType::Image)
    }

    /// Returns `true` if the current [`AssetType`] is a video
    pub fn is_video(&self) -> bool {
        matches!(self, AssetType::Video)
    }
}

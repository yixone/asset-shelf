auto_derived_ty! {
    /// Represents the lifecycle state of an asset
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
    /// Generalized type of the asset's media
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

auto_derived_ty! {
    no_sqlx;

    /// Specifies the field used to sort assets
    #[derive(Debug)]
    pub enum AssetSortBy {
        /// Sort by creation date
        CreatedAt,
        /// Sort by last modified date
        UpdatedAt,
        /// Sort by original file size
        FileSize,
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AssetError {
    /// The new asset is uploading without a media file
    NewAssetWithoutMedia,

    /// The asset is marked as deleted
    /// and cannot be modified until restored
    AssetDeleted,

    /// Asset file format not supported
    UnsupportedType,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MediaError {
    /// The media exceeds the maximum allowed object size in the storage
    TooLargeMedia { max_size: usize },
}

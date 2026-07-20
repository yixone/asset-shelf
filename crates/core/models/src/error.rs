#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AssetError {
    /// The new asset is uploading without a media file
    NewAssetWithoutMedia,

    /// The asset is marked as deleted
    /// and cannot be modified until restored
    AssetDeleted,
}

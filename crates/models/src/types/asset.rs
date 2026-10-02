/// Variant of an asset file
///
/// May represent the original media file or a generated
/// derivative optimized for a specific use case
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetFileVariant {
    /// Original media file uploaded by the user
    Original,

    /// Low-resolution preview for quick asset display
    Thumbnail,
}

impl AssetFileVariant {
    /// Returns the file variant as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetFileVariant::Original => "original",
            AssetFileVariant::Thumbnail => "thumbnail",
        }
    }
}

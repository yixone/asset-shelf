/// Variant of a media file
///
/// May represent the original media file or a generated
/// derivative optimized for a specific use case
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug)]
pub enum MediaVariant {
    /// Original media file uploaded by the user
    Original,

    /// Low-resolution preview for quick asset display
    Thumbnail,

    /// Short looped video preview generated from the original video
    LoopPreview,
}

impl MediaVariant {
    /// Returns the media variant as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaVariant::Original => "original",
            MediaVariant::Thumbnail => "thumbnail",
            MediaVariant::LoopPreview => "loop_preview",
        }
    }
}

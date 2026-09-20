/// Represents MIME type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum MimeKind {
    /// Image formats (PNG, JPEG, GIF, WebP, etc.)
    Image,
    /// Video formats (MP4, WebM, AVI, etc.)
    Video,
    /// No specific kind assigned
    Unknown,
}

impl MimeKind {
    /// Returns `true` if the current [`MimeKind`] is an image
    pub fn is_image(&self) -> bool {
        *self == MimeKind::Image
    }

    /// Returns `true` if the current [`MimeKind`] is a video
    pub fn is_video(&self) -> bool {
        *self == MimeKind::Video
    }

    /// Returns `true` if the current [`MimeKind`] is unknown
    pub fn is_unknown(&self) -> bool {
        *self == MimeKind::Unknown
    }
}

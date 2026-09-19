/// Represents MIME type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimeKind {
    /// Image formats (PNG, JPEG, GIF, WebP, etc.)
    Image,
    /// Video formats (MP4, WebM, AVI, etc.)
    Video,
}

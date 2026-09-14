//! Shared types related to media files and the domain

/// Generalized media type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    /// Image-type media
    Image,
    /// Video-type media
    Video,
}

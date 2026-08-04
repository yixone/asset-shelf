//! mime kind - generalized mime type

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MimeKind {
    Image,
    Video,
}

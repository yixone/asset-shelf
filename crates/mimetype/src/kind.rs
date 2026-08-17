//! mime kind - generalized mime type

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum MimeKind {
    Image,
    Video,
}

impl MimeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MimeKind::Image => "image",
            MimeKind::Video => "video",
        }
    }
}

impl std::fmt::Display for MimeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

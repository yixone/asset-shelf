#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum MediaVariant {
    #[default]
    Original,
    Thumbnail,
    LoopPreview,
}

impl MediaVariant {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MediaVariant::Original => "original",
            MediaVariant::Thumbnail => "thumbnail",
            MediaVariant::LoopPreview => "loop_preview",
        }
    }
}

impl std::fmt::Display for MediaVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

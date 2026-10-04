use crate::id::FileGroupId;

/// Identifies a [`File`] by its owning [`FileGroup`] and variant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileKey(FileGroupId, FileVariant);

impl FileKey {
    /// Creates a new [`FileKey`]
    pub fn new(file_group_id: FileGroupId, file_variant: FileVariant) -> Self {
        Self(file_group_id, file_variant)
    }

    /// Returns a reference to the identifier of the owning [`FileGroup`]
    pub fn group_id(&self) -> &FileGroupId {
        &self.0
    }

    /// Returns the file variant
    pub fn variant(&self) -> FileVariant {
        self.1
    }
}

/// Variant of an media file
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
pub enum FileVariant {
    /// Original media file uploaded by the user
    Original,

    /// Low-resolution preview for quick asset display
    Thumbnail,
}

impl FileVariant {
    /// Returns the file variant as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Thumbnail => "thumbnail",
        }
    }
}

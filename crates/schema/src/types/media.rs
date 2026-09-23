use crate::id::MediaId;

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

/// A key identifying the media file in the storage
///
/// `MediaFileKey` is independent of the underlying storage backend and does not
/// represent a file system path. It identifies a stored media file within
/// the application's storage namespace
///
/// The key cannot be modified through its public API
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[derive(Debug, Clone, PartialEq)]
pub struct MediaFileKey(String);

const KEY_SEP: char = '/';

impl MediaFileKey {
    /// Returns the storage key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Appends a path segment to the storage key
    pub(crate) fn push<S>(mut self, path: S) -> Self
    where
        S: AsRef<str>,
    {
        self.0.push(KEY_SEP);
        self.0.push_str(path.as_ref());

        self
    }
}

impl AsRef<str> for MediaFileKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Generates storage keys for media files
pub struct MediaStorageLayout;

impl MediaStorageLayout {
    /// Generates a [`MediaFileKey`] using the `v1` storage layout
    ///
    ///
    pub fn v1(id: &MediaId, variant: MediaVariant) -> MediaFileKey {
        MediaFileKey(shard(id.to_string(), 2)).push(variant.as_str())
    }
}

/// Applies path sharding, transforming: `abcdef` into `ab/cd/abcdef`
fn shard(path: impl AsRef<str>, steps: usize) -> String {
    let path = path.as_ref();

    let mut res = String::with_capacity(path.len() + steps * 3);

    for i in 0..steps {
        let idx = 2 * i;
        if idx + 2 > path.len() {
            break;
        }
        res.push_str(&path[idx..idx + 2]);
        res.push(KEY_SEP);
    }

    res.push_str(path);
    res
}

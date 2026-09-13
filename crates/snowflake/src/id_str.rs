//! Contains the definition and implementations for an string-based `Snowflake Id`

use crate::SnowflakeId;

/// [`SnowflakeId`] converted to a string via [`base62`] encoding
///
/// For more information, see the [`SnowflakeId`] documentation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnowflakeIdStr(pub(crate) String);

impl SnowflakeIdStr {
    /// Returns the [`SnowflakeIdStr`] as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<SnowflakeId> for SnowflakeIdStr {
    fn from(id: SnowflakeId) -> Self {
        let b62 = base62::encode(id.0 as u64);
        SnowflakeIdStr(b62)
    }
}

impl TryFrom<String> for SnowflakeIdStr {
    type Error = base62::B62Error;

    fn try_from(str: String) -> Result<Self, Self::Error> {
        if base62::is_valid_base62(&str) {
            Ok(Self(str))
        } else {
            Err(base62::B62Error::InvalidBase62)
        }
    }
}

impl std::fmt::Display for SnowflakeIdStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

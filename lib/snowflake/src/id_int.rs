//! Contains the definition and implementations for an integer-based `Snowflake Id`

/// 64-bit unique identifier used in distributed computing
///
/// ### Id parts
/// SnowflakeID consists of a `41-bit` timestamp, an `8-bit` node ID, and a `14-bit` sequence number
///
/// `[ (0) ]` | `[ == timestamp (41) == ]` | `[ node_id (8) ]` | `[ = sequence (14) = ]`
///
/// The sign bit of the integer is always zero, so the value of id is always greater than zero
///
/// ### Usage
///
/// ```
/// use snowflake::SnowflakeGenerator;
///
/// // Define a generator for Snowflake ID generation
/// let generator = SnowflakeGenerator::new(0);
///
/// // Generates an ID using the generator
/// let id = generator.get_id();
/// ```
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnowflakeId(pub(crate) i64);

impl SnowflakeId {
    /// Returns the [`SnowflakeId`] as an [`i64`]
    pub fn as_int(&self) -> i64 {
        self.0
    }
}

impl<T> From<T> for SnowflakeId
where
    T: Into<i64>,
{
    fn from(int: T) -> Self {
        SnowflakeId(int.into())
    }
}

impl std::fmt::Display for SnowflakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

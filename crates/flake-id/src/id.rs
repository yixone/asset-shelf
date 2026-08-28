/// Flake ID
///
/// ### About
/// Flake ID - an identifier inspired by Snowflake ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlakeId(pub i64);

impl std::fmt::Display for FlakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

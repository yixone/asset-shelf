/// Newtype for perceptual hash
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerceptualHash(pub(crate) i64);

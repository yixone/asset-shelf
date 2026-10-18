/// Newtype for perceptual hash
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerceptualHash(pub(crate) i64);

impl PerceptualHash {
    /// Creates a new [`PerceptualHash`]
    pub fn new(hash: i64) -> Self {
        PerceptualHash(hash)
    }
}

/// Newtype for SHA256 checksum
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sha256Checksum(pub(crate) [u8; 32]);

impl Sha256Checksum {
    /// Creates a new [`Sha256Checksum`]
    pub fn new(buffer: [u8; 32]) -> Self {
        Sha256Checksum(buffer)
    }

    /// Returns the `hex` representation of the checksum
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

use crate::FlakeId;

/// [`FlakeId`] converted to a string via base62 encoding
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlakeIdStr(pub String);

impl std::fmt::Display for FlakeIdStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<FlakeId> for FlakeIdStr {
    fn from(id: FlakeId) -> Self {
        let b62 = base62::encode(id.0 as u64);
        FlakeIdStr(b62)
    }
}

mod base62 {
    pub const CHARSET: [u8; 62] =
        *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    pub fn encode(mut i: u64) -> String {
        if i == 0 {
            return "0".into();
        }

        let mut out = Vec::new();

        while i > 0 {
            out.push(CHARSET[(i % 62) as usize] as char);
            i /= 62;
        }

        out.into_iter().rev().collect()
    }
}

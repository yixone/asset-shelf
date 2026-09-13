//! Tools for working with Base62 encoding
//!
//! ### Usage:
//! ```
//! use base62;
//!
//! // Base62 encoding
//! let encoded = base62::encode(4252);
//!
//! // Base62 decoding
//! let decoded = base62::decode("A8z1").unwrap();
//!
//! // Generating random 16-character base62 strings
//! let random = base62::random_str(16);
//! ```

/// The set of characters used by the Base62 encoding
const CHARSET: [u8; 62] = *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encodes an int to Base62
///
/// ### Usage
/// ```
/// use base62;
///
/// let encoded = base62::encode(42175219);
/// ```
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

/// Decodes a Base62 string or returns a [`B62Error`]
///
/// ### Usage
/// ```
/// use base62;
///
/// let encoded = base62::encode(42);
/// let decoded = base62::decode(&encoded).unwrap();
///
/// assert_eq!(decoded, 42);
/// ```
pub fn decode(str: impl AsRef<str>) -> Result<u64, B62Error> {
    let mut num = 0u64;

    for c in str.as_ref().chars() {
        let next_dig = if c.is_ascii_digit() {
            (c as u8 - b'0') as u64
        } else if c.is_ascii_uppercase() {
            10 + (c as u8 - b'A') as u64
        } else if c.is_ascii_lowercase() {
            36 + (c as u8 - b'a') as u64
        } else {
            return Err(B62Error::InvalidBase62);
        };

        let n = num
            .checked_mul(62)
            .and_then(|v| v.checked_add(next_dig))
            .ok_or(B62Error::Overflow)?;

        num = n;
    }

    Ok(num)
}

/// Returns `true` if the provided string is a valid base62 string
pub fn is_valid_base62(str: impl AsRef<str>) -> bool {
    str.as_ref()
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || c.is_ascii_lowercase())
}

/// Generates a random base62 string of arbitrary length
///
/// /// ### Usage
/// ```
/// use base62;
///
/// let random = base62::random_str(16);
/// assert_eq!(random.len(), 16);
/// ```
pub fn random_str(len: usize) -> String {
    // Generates a random string using the standard `rng`
    random_str_with(len, rand::rng())
}

/// Generates a random base62 string of the specified length using the provided [`Rng`]
///
/// ### Usage
/// ```
/// use rand;
///
/// use base62;
///
/// let random = base62::random_str_with(16, rand::rng());
/// assert_eq!(random.len(), 16);
/// ```
pub fn random_str_with<R>(len: usize, rng: R) -> String
where
    R: rand::RngExt + rand::Rng,
{
    // Return an empty string immediately if a zero length is received
    if len == 0 {
        return String::new();
    }

    // Creates a character buffer of a specified length
    let mut out = Vec::with_capacity(len);

    // Creates an iterator of random bytes from the provided `rng`
    let b_gen = rng.random_iter::<u8>();

    // Generates random bytes until the required length is reached
    for b in b_gen {
        // Rejects bytes that fall outside the range
        if b >= 62 * 4 {
            continue;
        }

        // Inserts a byte as a base62 character into the string
        out.push(CHARSET[(b % 62) as usize] as char);

        // Stops generation if the string has reached the required length
        if out.len() == len {
            break;
        }
    }

    // Returns the character buffer as a string
    out.into_iter().collect()
}

/// Base62 processing error
#[derive(Debug)]
pub enum B62Error {
    /// Overflow error while decoding a base62 string
    Overflow,

    /// The resulting string is not a valid base62 string
    InvalidBase62,
}

impl std::fmt::Display for B62Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for B62Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode() {
        let encoded = encode(42);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, 42);
    }

    #[test]
    fn return_decode_error_for_invalid_b62() {
        let err = decode("abc=_++*").unwrap_err();
        assert!(matches!(err, B62Error::InvalidBase62));
    }

    #[test]
    fn generate_rand_b62() {
        const LENGTHS: &[usize] = &[8, 12, 36, 52, 64, 128, 256];
        assert!(LENGTHS.iter().all(|&v| random_str(v).len() == v));
    }
}

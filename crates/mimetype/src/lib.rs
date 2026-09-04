//! mimetype — types and helpers for guessing MIME types based on magic bytes

#[macro_use]
pub mod macros;

pub mod kind;

pub use crate::kind::MimeKind;

struct MimePattern {
    mime: MimeType,
    matcher: fn(&[u8]) -> bool,
}

define_mimes! {
    Jpeg, "image/jpeg", [
        (0, [0xFF, 0xD8, 0xFF, 0xDB]),
        (0, [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01]),
        (0, [0xFF, 0xD8, 0xFF, 0xEE]),
        (0, [0xFF, 0xD8, 0xFF, 0xE1, ?, ?, 0x45, 0x78, 0x69, 0x66, 0x00, 0x00]),
        (0, [0xFF, 0xD8, 0xFF, 0xE0])
    ];
    Png, "image/png", [
        (0, [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    ];
    Bmp, "image/bmp", [
        (0, [0x42, 0x4D])
    ];
    Webp, "image/webp", [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x57, 0x45, 0x42, 0x50])
    ];
    Gif, "image/gif", [
        (0, [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]),
        (0, [0x47, 0x49, 0x46, 0x38, 0x39, 0x61])
    ];
    Mp4, "video/mp4", [
        (4, [0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D]),
        (4, [0x66, 0x74, 0x79, 0x70, 0x4D, 0x53, 0x4E, 0x56])
    ];
    Avi, "video/x-msvideo", [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x41, 0x56, 0x49, 0x20])
    ];
    Webm, "video/webm", [
        (0, [0x1A, 0x45, 0xDF, 0xA3])
    ];
}

generate_ptree! {
    0x42 => [Bmp] as _PB_42;
    0x47 => [Gif] as _PB_47;
    0x52 => [Webp, Avi] as _PB_52;
    0x89 => [Png] as _PB_89;
    0x1A => [Webm] as _PB_1A;
    0xFF => [Jpeg] as _PB_FF;
}

impl MimeType {
    /// Guesses and returns the [`MimeType`] using magic bytes slice,
    /// otherwise returns [`GuessMimetypeError`] error
    pub fn guess(slice: &[u8]) -> Result<Self, GuessMimetypeError> {
        if slice.is_empty() {
            return Err(GuessMimetypeError);
        }

        // Tries to guess the mimetype using a prefix tree
        let prefix_byte = slice[0] as usize;
        let prefix = PREFIX_TREE[prefix_byte];
        if !prefix.is_empty() {
            for p in prefix {
                if (p.matcher)(slice) {
                    return Ok(p.mime);
                }
            }
        }

        // Fallback
        for p in PATTERNS {
            if (p.matcher)(slice) {
                return Ok(p.mime);
            }
        }

        // If the guess fails returns an error
        Err(GuessMimetypeError)
    }

    pub fn kind(&self) -> MimeKind {
        match self {
            MimeType::Jpeg | MimeType::Png | MimeType::Bmp | MimeType::Webp | MimeType::Gif => {
                MimeKind::Image
            }
            MimeType::Mp4 | MimeType::Avi | MimeType::Webm => MimeKind::Video,
        }
    }

    pub fn is_video(&self) -> bool {
        let kind = self.kind();
        kind == MimeKind::Video
    }
}

#[derive(Debug)]
pub struct GuessMimetypeError;

impl std::fmt::Display for GuessMimetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for GuessMimetypeError {}

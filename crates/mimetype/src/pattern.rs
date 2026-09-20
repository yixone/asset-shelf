use crate::MimeType;

/// Defines a matcher closure for the MimeType signature
macro_rules! pattern_matcher {
    ($val: expr, $ex: expr, $idx: expr, ? $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex,
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr, $pattern: literal $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex && ($val[$idx] == $pattern),
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr) => {
        $val.len() >= ($idx) && $ex
    };
}

/// Defines a list of patterns for guessing the MIME type
macro_rules! patterns {
    (
        $(
            $id: ident, $p_id: ident => [ $( ( $offset: expr, [ $( $tokens:tt ),* ] ) ) | + ]
        );* $(;)?
    ) => {
        $(
            pub const $p_id: &MimePattern = &MimePattern {
                mime: $crate::MimeType::$id,
                matcher: |b| $( pattern_matcher!(b, !b.is_empty(), $offset, $($tokens),*) )||+,
            };
        )*

        pub const PATTERNS: &[&MimePattern] = &[
            $( $p_id ),*
        ];

        impl $crate::MimeType {
            pub fn pattern(&self) -> Option<&'static MimePattern> {
                match self {
                    $(
                        Self::$id => Some($p_id),
                    )*
                    Self::OctetStream => None
                }
            }
        }
    };
}

/// Pattern for guessing the [`MimeType`] based on the magic bytes signature
pub struct MimePattern {
    pub mime: MimeType,
    pub matcher: fn(&[u8]) -> bool,
}

patterns! {
    // -- Image patterns --
    Jpeg, JPEG_PATTERN => [
        (0, [0xFF, 0xD8, 0xFF, 0xDB]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01]) |
        (0, [0xFF, 0xD8, 0xFF, 0xEE]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE1, ?, ?, 0x45, 0x78, 0x69, 0x66, 0x00, 0x00]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE0])
    ];
    Png, PNG_PATTERN => [
        (0, [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    ];
    Bmp, BMP_PATTERN => [
        (0, [0x42, 0x4D])
    ];
    Webp, WEBP_PATTERN => [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x57, 0x45, 0x42, 0x50])
    ];
    Gif, GIF_PATTERN => [
        (0, [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]) |
        (0, [0x47, 0x49, 0x46, 0x38, 0x39, 0x61])
    ];

    // -- Video patterns --
    Mp4, MP4_PATTERN => [
        (4, [0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D]) |
        (4, [0x66, 0x74, 0x79, 0x70, 0x4D, 0x53, 0x4E, 0x56])
    ];
    Avi, AVI_PATTERN => [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x41, 0x56, 0x49, 0x20])
    ];
    Webm, WEBM_PATTERN => [
        (0, [0x1A, 0x45, 0xDF, 0xA3])
    ];
}

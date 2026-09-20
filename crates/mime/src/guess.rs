use crate::{MimeType, pattern::*};

/// Builds a prefix tree to speed up MIME type detection
macro_rules! ptree {
    (
        $(
            $prefix: literal => [
                $( $mime_pattern: expr ),+
            ] as $bucket: ident;
        )+
    ) => {
        $(
            const $bucket: &[&MimePattern] = &[$( $mime_pattern ),+];
        )+

        static PREFIX_TREE: [&[&MimePattern]; 256] = {
            const EMPTY: &[&MimePattern] = &[];
            let mut arr = [EMPTY;256];
            $(
                arr[$prefix] = $bucket;
            )+
            arr
        };
    };
}

ptree! {
    0x42 => [BMP_PATTERN] as _PB_42;
    0x47 => [GIF_PATTERN] as _PB_47;
    0x52 => [WEBM_PATTERN, AVI_PATTERN] as _PB_52;
    0x89 => [PNG_PATTERN] as _PB_89;
    0x1A => [WEBM_PATTERN] as _PB_1A;
    0xFF => [JPEG_PATTERN] as _PB_FF;
}

/// Attempts to identify the [`MimeType`] of the given byte slice
///
/// The [`MimeType`] is guessing from the file's content using known
/// magic-bytes signatures. If the content does not match any known signature,
/// [`MimeType::OctetStream`] is returned
///
/// Returns `None` if the input is empty
pub fn guess_mime(slice: impl AsRef<[u8]>) -> Option<MimeType> {
    let magic = slice.as_ref();

    if magic.is_empty() {
        return None;
    }

    // Tries to guess the mimetype using a prefix tree
    let prefix_byte = magic[0] as usize;
    let prefix = PREFIX_TREE[prefix_byte];
    if !prefix.is_empty() {
        for p in prefix {
            if (p.matcher)(magic) {
                return Some(p.mime);
            }
        }
    }

    // Fallback
    for p in PATTERNS {
        if (p.matcher)(magic) {
            return Some(p.mime);
        }
    }

    // If the guess fails returns an `OctetStream`
    Some(MimeType::OctetStream)
}

#[cfg(test)]
mod tests {
    use crate::MimeType;

    #[test]
    fn guess_mime() {
        const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        let mime = super::guess_mime(PNG_MAGIC).unwrap();
        assert_eq!(mime, MimeType::Png);
    }

    #[test]
    fn guess_mime_with_placeholder() {
        const WEBP_MAGIC: &[u8] = &[
            0x52, 0x49, 0x46, 0x46, 0x00, 0x42, 0xFF, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];

        let mime = super::guess_mime(WEBP_MAGIC).unwrap();
        assert_eq!(mime, MimeType::Webp);
    }

    #[test]
    fn return_octet_stream_on_unknown() {
        const MAGIC: &[u8] = &[0xFF];
        let mime = super::guess_mime(MAGIC).unwrap();
        assert_eq!(mime, MimeType::OctetStream);
    }

    #[test]
    fn return_none_on_empty() {
        assert!(super::guess_mime([]).is_none());
    }
}

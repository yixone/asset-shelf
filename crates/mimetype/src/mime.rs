use crate::MimeKind;

macro_rules! mime {
    (
        $(
            $( #[$meta: meta] )*
            $kind: ident, $id: ident, $mimetype: literal, $ext: literal
        );* $(;)?
    ) => {
        /// Represents the MIME type of a file
        ///
        /// Identifies the format and general content type of the file
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum MimeType {
            $( $( #[$meta] )* $id ),*
        }

        impl MimeType {
            /// Returns the current [`MimeType`] as an `RFC 6838` string
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$id => $mimetype),*
                }
            }

            /// Returns the file extension for the current [`MimeType`]
            pub const fn extension(&self) -> &'static str {
                match self {
                    $(Self::$id => $mimetype),*
                }
            }

            /// Returns the [`MimeKind`] of the current [`MimeType`]
            pub const fn kind(&self) -> $crate::MimeKind {
                match self {
                    $(Self::$id => $crate::MimeKind::$kind),*
                }
            }
        }
    };
}

mime! {
    /// JPEG image
    Image, Jpeg, "image/jpeg","jpeg";
    /// PNG image
    Image, Png, "image/png", "png";
    /// BMP image
    Image, Bmp, "image/bmp", "bmp";
    /// WebP image
    Image, Webp, "image/webp", "webp";
    /// GIF image
    Image, Gif, "image/gif", "gif";

    /// MP4 video
    Video, Mp4, "video/mp4", "mp4";
    /// AVI video
    Video, Avi, "video/x-msvideo", "avi";
    /// WebM video
    Video, Webm, "video/webm", "webm";
}

impl MimeType {
    /// Returns `true` if the current [`MimeType`] is an image
    pub fn is_image(&self) -> bool {
        self.kind() == MimeKind::Image
    }

    /// Returns `true` if the current [`MimeType`] is a video
    pub fn is_video(&self) -> bool {
        self.kind() == MimeKind::Video
    }
}

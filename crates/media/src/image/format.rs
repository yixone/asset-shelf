/// List of encoding formats
///
/// Does not reflect all available formats!
pub enum ImageFormat {
    /// An Image in PNG Format
    Png,
    /// An Image in JPEG Format
    Jpeg,
    /// An Image in WEBP Format with encode quality (0-100)
    WebP { quality: usize },
}

impl From<ImageFormat> for image::ImageFormat {
    fn from(i: ImageFormat) -> Self {
        match i {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::WebP { .. } => image::ImageFormat::WebP,
        }
    }
}

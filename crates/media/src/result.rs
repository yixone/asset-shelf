use image::ImageError;

#[derive(Debug)]
pub enum MediaError {
    ImgBackendError(ImageError),
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MediaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MediaError::ImgBackendError(e) => Some(e),
        }
    }
}

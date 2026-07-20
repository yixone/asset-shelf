use image::ImageError;

#[derive(Debug)]
pub enum MediaError {
    ImgBackendError(ImageError),
    Io(std::io::Error),
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
            MediaError::Io(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for MediaError {
    fn from(e: std::io::Error) -> Self {
        MediaError::Io(e)
    }
}

impl From<image::ImageError> for MediaError {
    fn from(e: image::ImageError) -> Self {
        MediaError::ImgBackendError(e)
    }
}

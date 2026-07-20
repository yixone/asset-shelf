pub mod features;
pub mod image;

pub mod result;

pub(crate) type Result<T> = std::result::Result<T, result::MediaError>;

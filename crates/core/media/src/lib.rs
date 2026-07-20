pub mod features;
pub mod image;

pub mod result;

pub use result::MediaError;

pub(crate) type Result<T> = std::result::Result<T, result::MediaError>;

#![allow(async_fn_in_trait)]

use actix_multipart::Field;
use futures::TryStreamExt;
use result::{Error, ErrorKind};
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

#[derive(Debug)]
pub enum MultipartParseError {
    ReadError,
    TooLargeString,
    InvalidUtf8,
}

impl std::fmt::Display for MultipartParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MultipartParseError {}

impl From<MultipartParseError> for Error {
    #[track_caller]
    fn from(e: MultipartParseError) -> Self {
        let kind = match e {
            MultipartParseError::ReadError | MultipartParseError::InvalidUtf8 => {
                ErrorKind::MalformedPayload
            }
            MultipartParseError::TooLargeString => ErrorKind::StringTooLong {
                max_size: MAX_STR_BUFFER_SIZE,
            },
        };
        Error::new(kind)
    }
}

pub trait FieldExt {
    /// Reads a [`Field`] into a UTF-8 string or returns an [`MultipartParseError`]
    ///
    /// Returns an error if the string size exceeds 8 MB
    async fn read_to_string(&mut self) -> Result<String, MultipartParseError>;

    /// Returns the [`Field`] as an object that implements [`AsyncRead`]
    fn into_async_reader(self) -> impl AsyncRead;
}

pub const MAX_STR_BUFFER_SIZE: usize = 8 * 1024 * 1024;

impl FieldExt for Field {
    async fn read_to_string(&mut self) -> Result<String, MultipartParseError> {
        let mut buf = Vec::with_capacity(4096);
        while let Some(chunk) = self
            .try_next()
            .await
            .map_err(|_| MultipartParseError::ReadError)?
        {
            buf.extend_from_slice(&chunk);
            if buf.len() > MAX_STR_BUFFER_SIZE {
                return Err(MultipartParseError::TooLargeString);
            }
        }
        String::from_utf8(buf).map_err(|_| MultipartParseError::InvalidUtf8)
    }

    fn into_async_reader(self) -> impl AsyncRead {
        StreamReader::new(self.map_err(|_| std::io::Error::other(MultipartParseError::ReadError)))
    }
}

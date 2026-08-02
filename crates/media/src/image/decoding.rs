use std::io::Cursor;

use result::{Result, error::ResultExt};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::image::Image;

pub struct ImageDecoder;

impl ImageDecoder {
    pub async fn from_async_read<R>(mut reader: R) -> Result<Image>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.to_app_err()?;

        let im_reader = image::ImageReader::new(Cursor::new(buf));
        let decoded = im_reader
            .with_guessed_format()
            .to_app_err()?
            .decode()
            .to_app_err()?;

        Ok(Image { inner: decoded })
    }
}

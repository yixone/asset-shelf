use std::{collections::VecDeque, io::Cursor};

use bytes::Bytes;
use image::{DynamicImage, imageops::FilterType};
use result::{Result, create_error, error::ResultExt};

use crate::image::{ImageFormat, ImageReader, features::ImageFeatures};

/// Opened and decoded image
pub struct Image {
    pub(crate) inner: DynamicImage,
}

impl Image {
    /// Returns the dimensions of the current image
    pub fn dimension(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    /// Returns a feature extractor for the given image
    pub fn features<'a>(&'a self) -> ImageFeatures<'a> {
        ImageFeatures { img: self }
    }

    ///  Returns an encoded preview with the specified width
    pub fn thumbnail(&self, n_width: u32) -> Image {
        let (w, h) = self.dimension();
        let n_height = (h * n_width) / w;

        Image {
            inner: self
                .inner
                .resize_exact(n_width, n_height, FilterType::Triangle),
        }
    }

    /// Encodes the image and returns a reader for it
    pub fn reader(self, format: ImageFormat) -> Result<ImageReader> {
        let mut buf = Vec::new();

        match format {
            ImageFormat::Png | ImageFormat::Jpeg => {
                self.inner
                    .write_to(Cursor::new(&mut buf), format.into())
                    .to_app_err()?;
            }
            ImageFormat::WebP { quality } => {
                let encoder = match webp::Encoder::from_image(&self.inner) {
                    Ok(e) => e,
                    Err(_) => {
                        return Err(create_error!(UnsupportedFileType));
                    }
                };

                let webp = encoder.encode(quality as f32);
                buf.extend(&*webp);
            }
        }

        let chunks = buf
            .chunks(1024 * 256)
            .map(Bytes::copy_from_slice)
            .collect::<VecDeque<_>>();

        Ok(ImageReader { buffered: chunks })
    }
}

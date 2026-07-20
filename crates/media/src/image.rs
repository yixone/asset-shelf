use std::{
    collections::VecDeque,
    io::Cursor,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use image::{DynamicImage, GenericImageView, Pixel, imageops::FilterType};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

use crate::{Result, features};

// `image::ImageFormat` re-export
pub use image::ImageFormat;

pub struct Image {
    inner: DynamicImage,
}

pub struct FeaturedImage {
    inner: DynamicImage,
}

pub struct ImageReader {
    buffered: VecDeque<Bytes>,
}

impl Image {
    pub async fn from_reader<R>(mut reader: R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let im_reader = image::ImageReader::new(Cursor::new(buf));
        let decoded = im_reader.with_guessed_format()?.decode()?;

        Ok(Image { inner: decoded })
    }

    // TODO: Optimize image endcoding pipeline
    pub async fn to_reader(self, format: ImageFormat) -> Result<ImageReader> {
        let mut buf = Vec::new();
        self.inner.write_to(Cursor::new(&mut buf), format)?;

        let chunks = buf
            .chunks(1024 * 256)
            .map(Bytes::copy_from_slice)
            .collect::<VecDeque<_>>();
        Ok(ImageReader { buffered: chunks })
    }

    pub fn dimension(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    pub fn thumbnail(&self, n_width: u32) -> Self {
        let (w, h) = self.dimension();
        let n_height = (h * n_width) / w;
        Image {
            inner: self
                .inner
                .resize_exact(n_width, n_height, FilterType::Triangle),
        }
    }

    pub fn prepare_features(self) -> FeaturedImage {
        FeaturedImage {
            inner: self.inner.resize_exact(32, 32, FilterType::Triangle),
        }
    }
}

impl FeaturedImage {
    pub fn p_hash(&self) -> i64 {
        let luma = self.inner.to_luma8();
        let matrix: [[u8; 32]; 32] = luma
            .rows()
            .map(|i| {
                i.map(|p| p.0[0])
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Prepared image must be 32 pixels heigh")
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Prepared image must be 32 pixels width");
        drop(luma);

        features::hash::p_hash(matrix)
    }

    pub fn a_hash(&self) -> i64 {
        let resized = self.inner.resize_exact(8, 8, FilterType::Triangle);
        let luma = resized.to_luma8();
        drop(resized);
        let pixels = luma
            .pixels()
            .map(|p| p.0[0])
            .collect::<Vec<_>>()
            .try_into()
            .expect("After preparation there should be 64 pixels");
        drop(luma);

        features::hash::a_hash(pixels)
    }

    pub fn avg_color(&self) -> (u8, u8, u8) {
        let resized = self.inner.resize_exact(1, 1, FilterType::Nearest);
        let [r, g, b] = resized.get_pixel(0, 0).to_rgb().0;
        (r, g, b)
    }
}

impl AsyncRead for ImageReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        let inner_buf = match self.buffered.front_mut() {
            Some(c) => c,
            None => return Poll::Ready(Ok(())),
        };
        let len = inner_buf.len().min(buf.remaining());
        let to_put = inner_buf.split_to(len);
        buf.put_slice(&to_put);

        if inner_buf.is_empty() {
            self.buffered.pop_front();
        }

        Poll::Ready(Ok(()))
    }
}

use media::image::{Image, ImageDecoder, ImageFormat, ImageReader};
use mimetype::MimeType;
use models::{media::MediaVariant, types::Color};
use result::Result;
use tokio::io::AsyncRead;

use super::ExtractedFeatures;

const THUMBNAIL_WIDTH: u32 = 400;
const THUMBNAIL_QUALITY: usize = 80;

pub struct GeneratedImageVariant {
    pub variant: MediaVariant,
    pub mimetype: MimeType,
    pub reader: ImageReader,
}

pub struct ImageProcessor {
    image: Image,
}

impl ImageProcessor {
    pub fn new(img: Image) -> Self {
        ImageProcessor { image: img }
    }

    pub async fn decode<R>(data: R) -> Result<Self>
    where
        R: AsyncRead + Send + Unpin,
    {
        let img = ImageDecoder::from_async_read(data).await?;
        Ok(ImageProcessor::new(img))
    }

    /// Extracts features from an image and returns them as [`ExtractedFeatures`]
    pub fn extract_features(&self) -> ExtractedFeatures {
        let (width, height) = self.image.dimension();

        let featured = self.image.features();
        let color = Color::from(featured.avg_color());
        let p_hash = featured.p_hash();
        let a_hash = featured.a_hash();

        ExtractedFeatures {
            a_hash,
            p_hash,
            color,
            width,
            height,
        }
    }

    /// Generates a thumbnail for the current image and returns it as a reader
    pub fn generate_thumbnail(&self) -> Result<GeneratedImageVariant> {
        let thumbnail = self
            .image
            .thumbnail(THUMBNAIL_WIDTH)
            .reader(ImageFormat::WebP {
                quality: THUMBNAIL_QUALITY,
            })?;

        Ok(GeneratedImageVariant {
            variant: MediaVariant::Thumbnail,
            mimetype: MimeType::Webp,
            reader: thumbnail,
        })
    }
}

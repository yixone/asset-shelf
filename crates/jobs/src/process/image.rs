use std::str::FromStr;

use media::image::{Image, ImageDecoder, ImageFormat, ImageReader};
use mimetype::MimeType;
use models::{assets::view::AssetView, media::MediaVariant, types::Color};
use result::{Result, error::ResultExt};
use storage::StoragePath;
use tokio::io::AsyncRead;

use crate::{
    JobContext,
    process::{
        ExtractedFeatures,
        storage_api::{get_original, store_image_variant},
    },
};

const THUMBNAIL_WIDTH: u32 = 400;
const THUMBNAIL_QUALITY: usize = 80;

pub struct GeneratedImageVariant {
    pub variant: MediaVariant,
    pub mimetype: MimeType,
    pub reader: ImageReader,
}

pub struct ImageProcessor {
    pub(crate) image: Image,
}

impl ImageProcessor {
    pub async fn decode<R>(data: R) -> Result<Self>
    where
        R: AsyncRead + Send + Unpin,
    {
        let img = ImageDecoder::from_async_read(data).await?;
        Ok(ImageProcessor { image: img })
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

pub async fn process_asset_as_image(ctx: &JobContext, asset: &AssetView) -> Result<()> {
    // Retrieves information about the original media file
    let original = get_original(ctx, asset.media_id()).await?;

    // Retrieves the original file from the storage
    let path = StoragePath::from_str(&original.storage_path).to_app_err()?;
    let file = ctx.storage.open(&path).await?;

    // Decodes the image from the original file
    let processor = ImageProcessor::decode(file).await?;

    // Checks which variants already exist for the specified asset
    let variants = asset.media_variants();

    // Generates and saves a thumbnail
    if !variants.contains(&MediaVariant::Thumbnail) {
        let thumbnail = processor.generate_thumbnail()?;
        store_image_variant(ctx, thumbnail, asset.media_id()).await?;
    }

    // Retrieves the basic image parameters and features
    let features = processor.extract_features();

    // Writes features to the database
    let patch = features.into();
    ctx.db.assets.update_features(asset.inner.id, patch).await?;

    Ok(())
}

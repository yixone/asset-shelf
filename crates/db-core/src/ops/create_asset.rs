use models::entities::{Asset, AssetFeatures, Media, MediaFile};
use result::Result;

use crate::{ops::Operation, types::InsertResult};

#[async_trait::async_trait]
pub trait CreateAssetOp: Operation {
    async fn insert_asset(&mut self, asset: &Asset) -> Result<InsertResult>;

    async fn insert_features(&mut self, features: &AssetFeatures) -> Result<InsertResult>;

    async fn insert_media(&mut self, media: &Media) -> Result<InsertResult>;

    async fn insert_media_file(&mut self, file: &MediaFile) -> Result<InsertResult>;
}

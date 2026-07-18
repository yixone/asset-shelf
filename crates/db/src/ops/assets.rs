use models::entities::Asset;

use crate::core::{Result, result::InsertResult};

pub trait AssetsOps {
    async fn insert_asset(&mut self, asset: &Asset) -> Result<InsertResult>;
}

pub trait AssetsQuery {}

pub trait AssetFeaturesRepo {}

pub trait AssetFeaturesQuery {}

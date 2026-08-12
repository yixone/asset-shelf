use models::entities::{Asset, AssetFeatures, MediaFile};

pub struct AssetQuery {
    pub inner: Asset,
    pub features: AssetFeatures,
    pub media: Vec<MediaFile>,
}

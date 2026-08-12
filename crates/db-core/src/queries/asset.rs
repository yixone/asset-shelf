use join::JoinBuilder;
use models::entities::{Asset, AssetFeatures, MediaFile};

#[derive(Debug, Clone)]
pub struct AssetQuery {
    pub inner: Asset,
    pub features: AssetFeatures,
    pub media: Vec<MediaFile>,
}

impl AssetQuery {
    pub fn from_domains(
        assets: Vec<Asset>,
        features: Vec<AssetFeatures>,
        media: Vec<MediaFile>,
    ) -> Vec<AssetQuery> {
        JoinBuilder::new(assets)
            .with(features, |a| a)
            .with_group(media, |(a, _)| a)
            .transform(|((a, af), mf)| (a, af, mf))
            .build_as(|(a, af, mf)| AssetQuery {
                inner: a,
                features: af,
                media: mf,
            })
    }
}

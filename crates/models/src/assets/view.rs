use std::collections::HashSet;

use join::{JoinBuilder, impl_joinable};

use crate::{
    assets::{Asset, AssetFeatures, similar::SimilarAsset},
    media::{MediaFile, MediaVariant},
    types::{AssetId, MediaId},
};

/// [`Asset`] view model
#[derive(Debug, Clone)]
pub struct AssetView {
    pub inner: Asset,
    pub features: AssetFeatures,
    pub media: Vec<MediaFile>,
}

impl AssetView {
    /// Returns the [`AssetId`] of this [`AssetView`]
    pub fn id(&self) -> AssetId {
        self.inner.id
    }

    /// Returns a reference to the [`MediaId`] of this [`AssetView`]
    pub fn media_id(&self) -> &MediaId {
        &self.inner.media_id
    }

    /// Returns the variants that have already been generated for the given [`AssetView`]
    pub fn media_variants(&self) -> HashSet<MediaVariant> {
        let mut variants = HashSet::with_capacity(self.media.len());
        for v in &self.media {
            variants.insert(v.variant);
        }
        variants
    }
}

/// View model for an [`Asset`] found
/// by "search by similar assets"
#[derive(Debug, Clone)]
pub struct SimilarAssetView {
    pub asset: AssetView,
    pub score: SimilarAsset,
}

impl AssetView {
    /// Assembles the [`AssetView`] from models
    pub fn from_models(
        assets: Vec<Asset>,
        features: Vec<AssetFeatures>,
        media: Vec<MediaFile>,
    ) -> Vec<AssetView> {
        JoinBuilder::new(assets)
            .with(features, |a| a)
            .with_group(media, |(a, _)| a)
            .transform(|((a, af), mf)| (a, af, mf))
            .build_as(AssetView::from)
    }
}

impl From<(Asset, AssetFeatures, Vec<MediaFile>)> for AssetView {
    fn from((a, af, m): (Asset, AssetFeatures, Vec<MediaFile>)) -> Self {
        AssetView {
            inner: a,
            features: af,
            media: m,
        }
    }
}

impl_joinable!(SimilarAsset[item.asset_id] with AssetView[inner.id] as AssetId);

impl SimilarAssetView {
    /// Assembles the [`SimilarAssetView`] from models
    pub fn from_models(s: Vec<SimilarAsset>, a: Vec<AssetView>) -> Vec<SimilarAssetView> {
        JoinBuilder::new(s)
            .with(a, |s| s)
            .build_as(SimilarAssetView::from)
    }
}

impl From<(SimilarAsset, AssetView)> for SimilarAssetView {
    fn from((s, a): (SimilarAsset, AssetView)) -> Self {
        SimilarAssetView { asset: a, score: s }
    }
}

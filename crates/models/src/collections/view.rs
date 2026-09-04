use crate::{
    assets::view::AssetView,
    collections::{Collection, CollectionAdditions, CollectionAsset},
    impl_joinable,
    join::JoinBuilder,
    types::AssetId,
};

/// [`Collection`] view model
#[derive(Debug, Clone)]
pub struct CollectionView {
    pub inner: Collection,
    pub add: CollectionAdditions,
}

/// View model for a collection item
#[derive(Debug, Clone)]
pub struct CollectionItemView {
    pub inner: CollectionAsset,
    pub asset: AssetView,
}

impl CollectionView {
    /// Assembles the [`CollectionView`] from models
    pub fn from_models(
        collections: Vec<Collection>,
        additions: Vec<CollectionAdditions>,
    ) -> Vec<CollectionView> {
        JoinBuilder::new(collections)
            .with(additions, |c| c)
            .build_as(CollectionView::from)
    }
}

impl From<(Collection, CollectionAdditions)> for CollectionView {
    fn from((c, ca): (Collection, CollectionAdditions)) -> Self {
        CollectionView { inner: c, add: ca }
    }
}

impl_joinable!(CollectionAsset[asset_id] with AssetView[inner.id] as AssetId);

impl CollectionItemView {
    /// Assembles the [`CollectionItemView`] from models
    pub fn from_models(
        items: Vec<CollectionAsset>,
        assets: Vec<AssetView>,
    ) -> Vec<CollectionItemView> {
        JoinBuilder::new(items)
            .with(assets, |i| i)
            .build_as(CollectionItemView::from)
    }
}

impl From<(CollectionAsset, AssetView)> for CollectionItemView {
    fn from((c, a): (CollectionAsset, AssetView)) -> Self {
        CollectionItemView { inner: c, asset: a }
    }
}

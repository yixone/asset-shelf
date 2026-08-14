use join::impl_joinable;

use crate::{
    assets::Asset,
    types::{AssetId, CollectionId},
};

pub mod addition;
pub mod assets;
pub mod model;
pub mod view;

pub use addition::CollectionAdditions;
pub use assets::CollectionAsset;
pub use model::Collection;

// `CollectionAsset` relations
impl_joinable!(CollectionAsset[asset_id] with Asset[id] as AssetId);
impl_joinable!(CollectionAsset[collection_id] with Collection[id] as CollectionId);

// `Collection` <-> `CollectionAdditions` relation
impl_joinable!(Collection[id] with CollectionAdditions[collection] as CollectionId);

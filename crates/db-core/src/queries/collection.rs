use models::entities::{Collection, CollectionAdditions, CollectionAsset};

use crate::queries::asset::AssetQuery;

pub struct CollectionQuery {
    pub inner: Collection,
    pub addition: CollectionAdditions,
}

pub struct CollectionItemQuery {
    pub inner: CollectionAsset,
    pub asset: AssetQuery,
}

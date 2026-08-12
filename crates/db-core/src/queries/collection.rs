use join::{JoinBuilder, Joinable, impl_joinable};
use models::{
    entities::{Collection, CollectionAdditions, CollectionAsset},
    types::AssetId,
};

use crate::queries::asset::AssetQuery;

#[derive(Debug, Clone)]
pub struct CollectionQuery {
    pub inner: Collection,
    pub addition: CollectionAdditions,
}

impl CollectionQuery {
    pub fn from_domains(
        collections: Vec<Collection>,
        additions: Vec<CollectionAdditions>,
    ) -> Vec<CollectionQuery> {
        JoinBuilder::new(collections)
            .with(additions, |c| c)
            .build_as(|(c, ca)| CollectionQuery {
                inner: c,
                addition: ca,
            })
    }
}

#[derive(Debug, Clone)]
pub struct CollectionItemQuery {
    pub inner: CollectionAsset,
    pub asset: AssetQuery,
}

impl_joinable!(CollectionAsset[asset_id] with AssetQuery[inner.id] as AssetId);

impl CollectionItemQuery {
    pub fn from_domains(
        collections: Vec<CollectionAsset>,
        assets: Vec<AssetQuery>,
    ) -> Vec<CollectionItemQuery> {
        JoinBuilder::new(collections)
            .with(assets, |ca| ca)
            .build_as(|(ca, a)| CollectionItemQuery {
                inner: ca,
                asset: a,
            })
    }
}

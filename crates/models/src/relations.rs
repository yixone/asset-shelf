use join::{Joinable, impl_joinable};

use crate::{entities::*, types::id::*};

// `Asset` <-> `Media` relations
impl_joinable!(Asset[media_id] with Media[id] as MediaId);
impl_joinable!(Asset[media_id] with MediaFile[media_id] as MediaId);

// `Asset` <-> `AssetFeatures` relations
impl_joinable!(Asset[id] with AssetFeatures[asset_id] as AssetId);

// `CollectionAsset` relations
impl_joinable!(CollectionAsset[asset_id] with Asset[id] as AssetId);
impl_joinable!(CollectionAsset[collection_id] with Collection[id] as CollectionId);

// `Collection` <-> `CollectionAdditions` relation
impl_joinable!(Collection[id] with CollectionAdditions[collection] as CollectionId);

// `Media` relations
impl_joinable!(Media[id] with MediaFile[media_id] as MediaId);

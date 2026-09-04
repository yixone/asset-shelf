use crate::{
    impl_joinable,
    media::{Media, MediaFile},
    types::{AssetId, MediaId},
};

pub mod features;
pub mod model;
pub mod similar;
pub mod state;
pub mod view;

pub use features::AssetFeatures;
pub use model::Asset;
pub use state::AssetState;

// `Asset` <-> `Media` relations
impl_joinable!(Asset[media_id] with Media[id] as MediaId);
impl_joinable!(Asset[media_id] with MediaFile[media_id] as MediaId);

// `Asset` <-> `AssetFeatures` relations
impl_joinable!(Asset[id] with AssetFeatures[asset_id] as AssetId);

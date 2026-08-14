use join::impl_joinable;

use crate::types::MediaId;

pub mod file;
pub mod model;
pub mod variant;
pub mod view;

pub use file::MediaFile;
pub use model::Media;
pub use variant::MediaVariant;

// `Media` relations
impl_joinable!(Media[id] with MediaFile[media_id] as MediaId);

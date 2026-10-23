use models::entities::{Asset, FileGroup};

/// An [`Asset`] together with its associated [`FileGroup`]
pub type AssetFull = (Asset, FileGroup);

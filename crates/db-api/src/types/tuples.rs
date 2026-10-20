use models::entities::{Asset, FileGroup};

/// An [`Asset`] together with its associated [`FileGroup`]
pub type AssetWithFiles = (Asset, FileGroup);

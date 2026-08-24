use result::Result;

use crate::repos::{asset::AssetRepository, collection::CollectionRepository};

pub async fn add_asset_to_collection<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    Ok(())
}

pub async fn remove_asset_from_collection<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    Ok(())
}

pub async fn get_collection_assets<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    Ok(())
}

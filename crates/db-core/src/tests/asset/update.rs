use flake_id::FlakeId;
use models::types::AssetId;

use crate::types::{UpdateResult, patch::AssetPatch};

use super::*;

pub async fn update_existing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let asset = insert_asset(&repo, "foo", &flake).await?;

    let patch = AssetPatch::new().title(Some("bar".to_string()));
    repo.update(asset.id, patch).await?;

    let asset = repo.get_by_id(asset.id).await?;
    assert_eq!(
        asset.inner.title,
        Some("bar".to_string()),
        "After the update, the asset name should change"
    );

    Ok(())
}

pub async fn return_not_found_on_missing<R: AssetRepository>(repo: R) -> Result<()> {
    let patch = AssetPatch::new().title(Some("bar".to_string()));

    let res = repo.update(AssetId(FlakeId(0)), patch).await?;
    assert!(
        matches!(res, UpdateResult::NotFound),
        "An attempt to update a non-existent asset must return `UpdateResult::NotFound`"
    );

    Ok(())
}

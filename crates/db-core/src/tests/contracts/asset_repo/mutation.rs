//! Tests for asset mutation contracts

use flake_id::FlakeId;
use models::types::AssetId;

use crate::types::patch::{AssetFeaturesPatch, AssetPatch};

use super::*;

/// Checks that the given [`AssetRepository`] correctly updates an existing [`Asset`]
/// and returns the correct [`UpdateResult`]
pub async fn update_existing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let asset = insert_asset(&repo, "foo", &flake).await?;

    let res = repo
        .update(asset.id, AssetPatch::new().title(Some("bar".to_string())))
        .await?;
    assert!(res.has_changes());

    let asset = repo.get_by_id(asset.id).await?;
    assert_eq!(asset.inner.title, Some("bar".to_string()),);

    Ok(())
}

/// Checks that the given [`AssetRepository`] correctly updates
/// [`AssetFeatures`] for the given [`Asset`]
pub async fn update_existing_features<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let asset = insert_asset(&repo, "foo", &flake).await?;

    let res = repo
        .update_features(asset.id, AssetFeaturesPatch::new().height(Some(1920)))
        .await?;
    assert!(res.has_changes());

    let asset = repo.get_by_id(asset.id).await?;
    assert_eq!(asset.features.height, Some(1920),);

    Ok(())
}

/// Tests returning [`UpdateResult::NotFound`]
/// when attempting to update a non-existent [`Asset`]
pub async fn return_not_found_when_updating_non_existent<R: AssetRepository>(
    repo: R,
) -> Result<()> {
    let res = repo
        .update(
            AssetId(FlakeId(0)),
            AssetPatch::new().title(Some("bar".to_string())),
        )
        .await?;

    assert!(res.no_changes());

    Ok(())
}

/// Checks that the given [`AssetRepository`] correctly deletes
/// an existing [`Asset`] and returns the appropriate [`DeleteResult`]
pub async fn delete_existing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let asset = insert_asset(&repo, "foo", &flake).await?;

    let res = repo.delete(asset.id).await?;
    assert!(res.has_changes());

    let err = repo.get_by_id(asset.id).await.unwrap_err();
    assert!(err.is_not_found());

    Ok(())
}

/// Tests returning [`DeleteResult::NoChanges`]
/// when attempting to delete a non-existent [`Asset`]
pub async fn return_no_changes_when_deleting_non_existent<R: AssetRepository>(
    repo: R,
) -> Result<()> {
    let res = repo.delete(AssetId::from(FlakeId(0))).await?;
    assert!(res.no_changes());

    Ok(())
}

use flake_id::FlakeId;
use models::types::AssetId;
use result::ErrorKind;

use super::*;

/// Tests that the existing asset is returned by ID
pub async fn get_existing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let asset = insert_asset(&repo, "foo", &flake).await?;

    let fetched_asset = repo.get_by_id(asset.id).await?;

    assert_eq!(
        fetched_asset.id(),
        asset.id,
        "The identifier of the inserted asset and the received asset must match"
    );

    assert_eq!(
        fetched_asset.inner.id, fetched_asset.features.asset_id,
        "The identifiers of the asset view model elements must be identical"
    );

    Ok(())
}

/// Tests that an [`ErrorKind::NotFound`] error is returned
/// when attempting to retrieve a non-existent [`Asset`]
pub async fn throw_error_on_missing<R: AssetRepository>(repo: R) -> Result<()> {
    let err = repo
        .get_by_id(AssetId::from(FlakeId(0)))
        .await
        .expect_err("A `get_by_id` request for a non-existent asset should return an error");

    assert!(
        matches!(err.kind(), ErrorKind::NotFound),
        "A `NotFound` error should be returned for a `get_by_id` on a non-existent asset"
    );

    Ok(())
}

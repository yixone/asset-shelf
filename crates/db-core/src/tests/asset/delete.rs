use flake_id::FlakeId;
use models::types::AssetId;
use result::ErrorKind;

use crate::types::DeleteResult;

use super::*;

pub async fn delete_existing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let asset = insert_asset(&repo, "foo", &flake).await?;

    repo.delete(asset.id).await?;

    let err = repo
        .get_by_id(asset.id)
        .await
        .expect_err("A `get_by_id` for a non-existent asset should return an error");

    assert!(
        matches!(err.kind(), ErrorKind::NotFound),
        "A `NotFound` error should be returned for a `get_by_id` on a non-existent asset"
    );

    Ok(())
}

pub async fn return_no_changes_on_missing<R: AssetRepository>(repo: R) -> Result<()> {
    let res = repo.delete(AssetId(FlakeId(0))).await?;

    assert!(
        matches!(res, DeleteResult::NoChanges),
        "An attempt to update a non-existent asset must return `UpdateResult::NotFound`"
    );

    Ok(())
}

use chrono::{Duration, Utc};
use flake_id::FlakeId;
use models::types::{AssetId, AssetsOrdering};

use crate::types::{Pagination, patch::AssetPatch};

use super::*;

/// Tests that the existing [`Asset`] is returned by ID
pub async fn get_an_existing_by_id<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let asset = insert_asset(&repo, "foo", &flake).await?;

    let fetched_asset = repo.get_by_id(asset.id).await?;

    assert_eq!(fetched_asset.id(), asset.id);
    assert_eq!(fetched_asset.inner.id, fetched_asset.features.asset_id);

    Ok(())
}

/// Tests that an [`ErrorKind::NotFound`] error is returned
/// when attempting to retrieve a non-existent [`Asset`]
pub async fn return_not_found_when_getting_a_non_existent_asset<R: AssetRepository>(
    repo: R,
) -> Result<()> {
    let err = repo.get_by_id(AssetId::from(FlakeId(0))).await.unwrap_err();
    assert!(err.is_not_found());

    Ok(())
}

/// Tests retrieving a list of assets marked as deleted
pub async fn get_deleted_list<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let first = {
        let asset = insert_asset(&repo, "foo", &flake).await?;
        let patch = AssetPatch::new().deleted_at(Some(Utc::now() + Duration::days(2)));
        repo.update(asset.id, patch).await?;
        asset
    };

    let second = {
        let asset = insert_asset(&repo, "bar", &flake).await?;
        let patch = AssetPatch::new().deleted_at(Some(Utc::now()));
        repo.update(asset.id, patch).await?;
        asset
    };

    let list = repo
        .get_deleted(Pagination::new(5, 0), AssetsOrdering::Newest)
        .await?;

    assert_eq!(list[0].id(), first.id);
    assert_eq!(list[1].id(), second.id);

    Ok(())
}

/// Tests the return of an empty list of assets
pub async fn list_empty<R: AssetRepository>(repo: R) -> Result<()> {
    let list = repo
        .list(Pagination::new(50, 0), AssetsOrdering::Newest)
        .await?;

    assert!(list.is_empty());

    Ok(())
}

/// Tests retrieving a list of assets in different orders:
/// - [`AssetsOrdering::Newest`]
/// - [`AssetsOrdering::Oldest`]
pub async fn list_ordered<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let first = insert_asset(&repo, "foo", &flake).await?;
    let second = insert_asset(&repo, "bar", &flake).await?;
    {
        let list_newest = repo
            .list(Pagination::new(50, 0), AssetsOrdering::Newest)
            .await?;
        assert_eq!(list_newest[0].id(), second.id);
        assert_eq!(list_newest[1].id(), first.id);
    }
    {
        let list_oldest = repo
            .list(Pagination::new(50, 0), AssetsOrdering::Oldest)
            .await?;
        assert_eq!(list_oldest[0].id(), first.id);
        assert_eq!(list_oldest[1].id(), second.id);
    }

    Ok(())
}

/// Tests getting a list of assets with [`Pagination`]
pub async fn list_with_pagination<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let first = insert_asset(&repo, "foo", &flake).await?;
    let second = insert_asset(&repo, "bar", &flake).await?;

    {
        let list = repo
            .list(Pagination::new(1, 0), AssetsOrdering::Newest)
            .await?;

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id(), second.id);
    }
    {
        let list = repo
            .list(Pagination::new(1, 1), AssetsOrdering::Newest)
            .await?;

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id(), first.id);
    }

    Ok(())
}

/// Tests the counting of the total number of assets
pub async fn count_assets<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let mut assets = Vec::new();

    assets.push(insert_asset(&repo, "foo", &flake).await?);
    assets.push(insert_asset(&repo, "bar", &flake).await?);
    assets.push(insert_asset(&repo, "bazz", &flake).await?);
    assets.push(insert_asset(&repo, "42", &flake).await?);

    let count = repo.count_total().await?;

    assert_eq!(count, assets.len() as u64);

    Ok(())
}

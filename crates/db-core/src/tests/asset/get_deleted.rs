use chrono::{Duration, Utc};
use models::types::AssetsOrdering;

use crate::types::{Pagination, patch::AssetPatch};

use super::*;

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

    assert_eq!(
        list[0].id(),
        first.id,
        "The asset that was deleted last should be returned first"
    );

    assert_eq!(
        list[1].id(),
        second.id,
        "The asset that was deleted second-to-last should be the first to return"
    );

    Ok(())
}

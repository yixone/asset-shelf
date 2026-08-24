use models::{media::MediaVariant, types::CollectionAssetsOrdering};

use crate::{tests::contracts::media_repo::insert_asset_with_files, types::Pagination};

use super::*;

pub async fn add_asset_to_collection<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let collection = prepare_collection(&flake, "test");
    cr.insert(&collection).await?;

    let a = insert_asset_with_files(&ar, "asset", &[MediaVariant::Original], &flake).await?;
    let item = cr
        .add_asset(flake.get_id_as(), collection.id, a.id())
        .await?;

    assert_eq!(item.collection_id, collection.id);
    assert_eq!(item.asset_id, a.id());

    let c_items = cr
        .get_items(
            collection.id,
            Pagination::new(50, 0),
            CollectionAssetsOrdering::Latest,
        )
        .await?;
    assert_eq!(c_items.len(), 1);

    Ok(())
}

pub async fn remove_asset_from_collection<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let collection = prepare_collection(&flake, "test");
    cr.insert(&collection).await?;

    let a = insert_asset_with_files(&ar, "asset", &[MediaVariant::Original], &flake).await?;
    let item = cr
        .add_asset(flake.get_id_as(), collection.id, a.id())
        .await?;

    cr.remove_asset(collection.id, item.id).await?;

    let c_items = cr
        .get_items(
            collection.id,
            Pagination::new(50, 0),
            CollectionAssetsOrdering::Latest,
        )
        .await?;

    assert!(c_items.is_empty());

    Ok(())
}

/// Tests deleting an `Asset` from a [`Collection`] it's not bound to.
/// Tests the scenario where a `rel_id` is passed that has no association with the target [`Collection`]
pub async fn do_not_remove_asset_for_unrelated_collection<
    AR: AssetRepository,
    CR: CollectionRepository,
>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let collection = prepare_collection(&flake, "test");
    cr.insert(&collection).await?;

    let collection2 = prepare_collection(&flake, "test2");
    cr.insert(&collection2).await?;

    let a = insert_asset_with_files(&ar, "asset", &[MediaVariant::Original], &flake).await?;
    let item = cr
        .add_asset(flake.get_id_as(), collection.id, a.id())
        .await?;

    let res = cr.remove_asset(collection2.id, item.id).await?;
    assert!(res.no_changes());

    let c_items = cr
        .get_items(
            collection.id,
            Pagination::new(50, 0),
            CollectionAssetsOrdering::Latest,
        )
        .await?;

    assert_eq!(c_items.len(), 1);
    assert_eq!(c_items[0].asset.inner, a.inner);
    assert_eq!(c_items[0].inner.id, item.id);

    Ok(())
}

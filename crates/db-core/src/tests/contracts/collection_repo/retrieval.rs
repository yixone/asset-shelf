use models::{media::MediaVariant, types::CollectionAssetsOrdering};

use crate::{tests::contracts::media_repo::insert_asset_with_files, types::Pagination};

use super::*;

pub async fn get_collection_with_additions<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let collection = prepare_collection(&flake, "test");
    cr.insert(&collection).await?;

    let mut assets = vec![];
    for _ in 0..4 {
        let a = insert_asset_with_files(&ar, "asset", &[MediaVariant::Original], &flake).await?;
        cr.add_asset(flake.get_id_as(), collection.id, a.id())
            .await?;

        assets.push(a);
    }

    let collection = cr.get_by_id(collection.id).await?;

    assert_eq!(collection.add.assets_count as usize, assets.len());

    let thumbnails = &collection.add.thumbnails;
    assert_eq!(thumbnails.len(), 3);

    Ok(())
}

pub async fn get_collection_items<AR: AssetRepository, CR: CollectionRepository>(
    ar: AR,
    cr: CR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let collection = prepare_collection(&flake, "test");
    cr.insert(&collection).await?;

    let mut assets = vec![];
    for _ in 0..4 {
        let a = insert_asset_with_files(&ar, "asset", &[MediaVariant::Original], &flake).await?;
        cr.add_asset(flake.get_id_as(), collection.id, a.id())
            .await?;

        assets.push(a);
    }

    let items = cr
        .get_items(
            collection.id,
            Pagination::new(50, 0),
            CollectionAssetsOrdering::Latest,
        )
        .await?;

    assert_eq!(items.len(), assets.len());

    assert_eq!(items[0].asset.id(), assets[3].id());

    assert_eq!(items[0].asset.id(), items[0].inner.asset_id);
    assert_eq!(collection.id, items[0].inner.collection_id);

    Ok(())
}

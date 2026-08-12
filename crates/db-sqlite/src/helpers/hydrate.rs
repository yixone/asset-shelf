use db_core::queries::{
    asset::AssetQuery,
    collection::{CollectionItemQuery, CollectionQuery},
    media::MediaQuery,
};
use models::entities::{Asset, Collection, CollectionAsset, Media};
use result::Result;

use crate::helpers::queries;

pub async fn hydrate_assets(
    assets: Vec<Asset>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<AssetQuery>> {
    let features_ids = assets.iter().map(|a| a.id).collect::<Vec<_>>();
    let features = queries::get_assets_features(&features_ids, &mut *exec).await?;

    let media_ids = assets
        .iter()
        .map(|a| a.media_id.clone())
        .collect::<Vec<_>>();
    let media = queries::get_media_files(&media_ids, &mut *exec).await?;

    let query = AssetQuery::from_domains(assets, features, media);

    Ok(query)
}

pub async fn hydrate_collections(
    collections: Vec<Collection>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<CollectionQuery>> {
    let ids = collections.iter().map(|c| c.id).collect::<Vec<_>>();
    let ca = queries::get_collections_additions(&ids, exec).await?;

    Ok(CollectionQuery::from_domains(collections, ca))
}

pub async fn hydrate_collection_assets(
    items: Vec<CollectionAsset>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<CollectionItemQuery>> {
    let assets_ids = items.iter().map(|c| c.asset_id).collect::<Vec<_>>();
    // Loads the list of asset domains
    let a = queries::get_assets(&assets_ids, &mut *exec).await?;
    let assets = hydrate_assets(a, exec).await?;

    // Assembles the model query
    let res = CollectionItemQuery::from_domains(items, assets);
    Ok(res)
}

pub async fn hydrate_media(
    media: Vec<Media>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<MediaQuery>> {
    let media_ids = media.iter().map(|m| m.id.clone()).collect::<Vec<_>>();

    let files = queries::get_media_files(&media_ids, &mut *exec).await?;

    let res = MediaQuery::from_domains(media, files);
    Ok(res)
}

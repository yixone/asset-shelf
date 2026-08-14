use models::{
    assets::{
        Asset,
        similar::SimilarAsset,
        view::{AssetView, SimilarAssetView},
    },
    collections::{
        Collection, CollectionAsset,
        view::{CollectionItemView, CollectionView},
    },
    media::{Media, view::MediaView},
};
use result::Result;

use crate::queries;

pub async fn hydrate_assets(
    assets: Vec<Asset>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<AssetView>> {
    let features_ids = assets.iter().map(|a| a.id).collect::<Vec<_>>();
    let features = queries::asset::get_assets_features(&features_ids, &mut *exec).await?;

    let media_ids = assets
        .iter()
        .map(|a| a.media_id.clone())
        .collect::<Vec<_>>();
    let media = queries::media::get_media_files(&media_ids, &mut *exec).await?;

    Ok(AssetView::from_models(assets, features, media))
}

pub async fn hydrate_similar(
    similar: Vec<SimilarAsset>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<SimilarAssetView>> {
    let assets_ids = similar.iter().map(|a| a.item.asset_id).collect::<Vec<_>>();
    let assets = queries::asset::get_assets(&assets_ids, &mut *exec).await?;

    let media_ids = assets
        .iter()
        .map(|a| a.media_id.clone())
        .collect::<Vec<_>>();
    let media = queries::media::get_media_files(&media_ids, &mut *exec).await?;

    let av = AssetView::from_models(
        assets,
        similar.iter().map(|f| f.item.clone()).collect(),
        media,
    );

    Ok(SimilarAssetView::from_models(av, similar))
}

pub async fn hydrate_collections(
    collections: Vec<Collection>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<CollectionView>> {
    let ids = collections.iter().map(|c| c.id).collect::<Vec<_>>();
    let ca = queries::collection::get_collections_additions(&ids, exec).await?;

    Ok(CollectionView::from_models(collections, ca))
}

pub async fn hydrate_collection_assets(
    items: Vec<CollectionAsset>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<CollectionItemView>> {
    let assets_ids = items.iter().map(|c| c.asset_id).collect::<Vec<_>>();
    // Loads the list of asset domains
    let a = queries::asset::get_assets(&assets_ids, &mut *exec).await?;
    let assets = hydrate_assets(a, exec).await?;

    // Assembles the model query
    Ok(CollectionItemView::from_models(items, assets))
}

pub async fn hydrate_media(
    media: Vec<Media>,
    exec: &mut sqlx::SqliteConnection,
) -> Result<Vec<MediaView>> {
    let media_ids = media.iter().map(|m| m.id.clone()).collect::<Vec<_>>();

    let files = queries::media::get_media_files(&media_ids, &mut *exec).await?;

    Ok(MediaView::from_models(media, files))
}

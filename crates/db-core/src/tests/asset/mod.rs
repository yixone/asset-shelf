//! Tests for the `Asset` domain CRUD operations

pub mod create_op;
pub mod delete;
pub mod get_by_id;
pub mod get_deleted;
pub mod get_for_processing;
pub mod get_from_similar;
pub mod get_similar_candidates;
pub mod list;
pub mod update;

use flake_id::FlakeIdGenerator;
use mimetype::MimeKind;
use models::{
    assets::{Asset, AssetFeatures},
    media::Media,
};
use result::Result;

use crate::repos::asset::AssetRepository;

pub async fn insert_asset<R: AssetRepository>(
    repo: &R,
    title: &str,
    flake: &FlakeIdGenerator,
) -> Result<Asset> {
    let (media, asset, asset_features) = prepare_asset(flake, title);
    insert_full_asset((&media, &asset, &asset_features), repo).await?;

    Ok(asset)
}

pub fn prepare_asset(flake: &FlakeIdGenerator, title: &str) -> (Media, Asset, AssetFeatures) {
    let media = Media::new(flake.get_id_as());
    let asset = Asset::new(
        flake.get_id_as(),
        media.id.clone(),
        MimeKind::Image,
        Some(title.into()),
        None,
        None,
    );
    let features = AssetFeatures::new(asset.id);

    (media, asset, features)
}

pub async fn insert_full_asset<R: AssetRepository>(
    (m, a, af): (&Media, &Asset, &AssetFeatures),
    repo: &R,
) -> Result<()> {
    let mut op = repo.create_op().await?;

    op.insert_media(m).await?;
    op.insert_asset(a).await?;
    op.insert_features(af).await?;

    op.commit().await
}

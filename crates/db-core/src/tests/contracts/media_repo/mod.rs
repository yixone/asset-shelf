use flake_id::FlakeIdGenerator;
use mimetype::MimeType;
use models::{
    assets::view::AssetView,
    media::{Media, MediaFile, MediaVariant},
    types::MediaId,
};
use result::Result;

use crate::{
    repos::{asset::AssetRepository, media::MediaRepository},
    tests::contracts::asset_repo::prepare_asset,
};

mod insertion;
mod mutation;
mod relations;
mod retrieval;

/// Tests all [`insertion`] contracts for the given [`MediaRepository`]
pub async fn test_media_insertion<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: MediaRepository,
{
    // Tests inserting two media files with the same variant into one parent media
    insertion::insert_media_with_same_files_variants(repo().await).await?;

    Ok(())
}

/// Tests all [`mutation`] contracts for the given [`MediaRepository`]
pub async fn test_media_mutation<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: MediaRepository,
{
    // Testing media file update
    mutation::update_existing_file(repo().await).await?;
    mutation::return_no_changes_when_updating_non_existent_file(repo().await).await?;

    // Testing media removal
    mutation::delete_existing(repo().await).await?;
    mutation::return_no_changes_when_deleting_non_existent(repo().await).await?;

    // Tests deleting a media file
    mutation::delete_existing_file(repo().await).await?;

    Ok(())
}

/// Tests all [`relations`] contracts for the given [`MediaRepository`]
pub async fn test_media_relations<F, MR, AR>(repo: F) -> Result<()>
where
    F: AsyncFn() -> (MR, AR),
    MR: MediaRepository,
    AR: AssetRepository,
{
    // Tests retrieving orphaned media
    {
        let (mr, _) = repo().await;
        relations::get_orphans(mr).await?;
    }
    {
        let (mr, ar) = repo().await;
        relations::get_empty_for_no_orphans(ar, mr).await?;
    }

    Ok(())
}

/// Tests all [`retrieval`] contracts for the given [`MediaRepository`]
pub async fn test_media_retrieval<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: MediaRepository,
{
    // Tests getting media variant
    retrieval::get_media_variant(repo().await).await?;
    retrieval::return_error_for_non_existent_variant(repo().await).await?;

    Ok(())
}

/// Creates a test [`Asset`] and inserts it with related files using the test [`AssetRepository`]
pub(crate) async fn insert_asset_with_files<R: AssetRepository>(
    repo: &R,
    title: &str,
    variants: &[MediaVariant],
    flake: &FlakeIdGenerator,
) -> Result<AssetView> {
    let (media, asset, asset_features) = prepare_asset(flake, title);
    let files = variants
        .iter()
        .map(|v| prepare_media_file(flake, &media.id, *v))
        .collect::<Vec<_>>();

    let mut op = repo.create_op().await?;
    op.insert_media(&media).await?;
    for f in &files {
        op.insert_media_file(f).await?;
    }
    op.insert_asset(&asset).await?;
    op.insert_features(&asset_features).await?;

    op.commit().await?;

    Ok(AssetView::from((asset, asset_features, files)))
}

/// Prepares [`Media`] for testing
pub(crate) fn prepare_media(flake: &FlakeIdGenerator) -> Media {
    Media::new(flake.get_id_as())
}

/// Prepares [`MediaFile`] for testing
pub(crate) fn prepare_media_file(
    flake: &FlakeIdGenerator,
    media: &MediaId,
    variant: MediaVariant,
) -> MediaFile {
    MediaFile::new(
        flake.get_id_as(),
        media.clone(),
        variant,
        "NONE".to_string(),
        0,
        MimeType::Jpeg,
        None,
    )
}

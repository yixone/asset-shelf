use flake_id::FlakeIdGenerator;
use mimetype::MimeType;
use models::media::{Media, MediaFile, MediaVariant};
use result::Result;

use crate::repos::{asset::AssetRepository, media::MediaRepository};

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
    insertion::insert_media_with_files(repo().await).await?;
    insertion::insert_media_with_same_files_variants(repo().await).await?;

    Ok(())
}

/// Tests all [`mutation`] contracts for the given [`MediaRepository`]
pub async fn test_media_mutation<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: MediaRepository,
{
    mutation::update_existing_file(repo().await).await?;
    mutation::return_not_found_when_updating_non_existent_file(repo().await).await?;

    mutation::delete_existing(repo().await).await?;
    mutation::return_no_changes_when_deleting_non_existent(repo().await).await?;

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
    retrieval::get_media_variant(repo().await).await?;
    retrieval::return_error_for_non_existent_variant(repo().await).await?;

    Ok(())
}

/// Prepares [`Media`] for testing
pub(crate) fn prepare_media(flake: &FlakeIdGenerator) -> Media {
    Media::new(flake.get_id_as())
}

/// Prepares [`MediaFile`] for testing
pub(crate) fn prepare_media_file(
    flake: &FlakeIdGenerator,
    media: &Media,
    variant: MediaVariant,
) -> MediaFile {
    MediaFile::new(
        flake.get_id_as(),
        media.id.clone(),
        variant,
        "NONE".to_string(),
        0,
        MimeType::Jpeg,
        None,
    )
}

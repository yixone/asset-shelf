use flake_id::FlakeIdGenerator;
use mimetype::MimeType;
use models::media::{Media, MediaFile, MediaVariant};
use result::Result;

use crate::repos::media::MediaRepository;

mod insertion;
mod mutation;
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

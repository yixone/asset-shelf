use flake_id::FlakeIdGenerator;
use models::media::MediaVariant;
use result::{ErrorKind, Result};

use crate::{
    repos::media::MediaRepository,
    tests::contracts::media_repo::{prepare_media, prepare_media_file},
};

/// Tests [`Media`] insertion with various [`MediaFile`]
pub async fn insert_media_with_files<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);
    let thumb = prepare_media_file(&flake, &media, MediaVariant::Thumbnail);

    repo.insert(&media).await?;

    repo.insert_file(&original).await?;
    repo.insert_file(&thumb).await?;

    let retrieved = repo.get_by_id(&media.id).await?;

    assert_eq!(retrieved.files.len(), 2);

    let variants = retrieved.media_variants();

    assert!(variants.contains(&MediaVariant::Original));
    assert!(variants.contains(&MediaVariant::Thumbnail));

    assert_eq!(retrieved.inner.id, media.id);

    Ok(())
}

/// Checks that the repository returns an error if two media files
/// with the same [`MediaVariant`] are inserted into the same [`Media`] group
pub async fn insert_media_with_same_files_variants<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);
    let original_2 = prepare_media_file(&flake, &media, MediaVariant::Original);

    repo.insert(&media).await?;

    repo.insert_file(&original).await?;
    {
        let res = repo.insert_file(&original_2).await.expect_err(
            "The repository should return an error when inserting two files with the same variants",
        );

        assert!(matches!(res.kind(), ErrorKind::AlreadyExists));
    }

    {
        let retrieved = repo.get_by_id(&media.id).await?;

        assert_eq!(retrieved.files.len(), 1);

        let variants = retrieved.media_variants();

        assert!(variants.contains(&MediaVariant::Original));
        assert!(!variants.contains(&MediaVariant::Thumbnail));
    }

    Ok(())
}

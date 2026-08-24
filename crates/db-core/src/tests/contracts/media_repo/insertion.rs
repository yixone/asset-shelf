use super::*;

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
        let err = repo.insert_file(&original_2).await.unwrap_err();
        assert!(err.is_conflict());
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

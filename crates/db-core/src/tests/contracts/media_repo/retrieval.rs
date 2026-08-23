use super::*;

/// Tests getting media variant
pub async fn get_media_variant<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);
    let thumb = prepare_media_file(&flake, &media, MediaVariant::Thumbnail);

    repo.insert(&media).await?;

    repo.insert_file(&original).await?;
    repo.insert_file(&thumb).await?;

    let file = repo.get_variant(&media.id, MediaVariant::Original).await?;

    assert_eq!(file.id, original.id);
    assert_eq!(file.variant, MediaVariant::Original);

    Ok(())
}

/// Tests returning an error when receiving a non-existent media variant
pub async fn return_error_for_non_existent_variant<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let media = prepare_media(&flake);

    let thumb = prepare_media_file(&flake, &media, MediaVariant::Thumbnail);

    repo.insert(&media).await?;

    repo.insert_file(&thumb).await?;

    let res = repo
        .get_variant(&media.id, MediaVariant::Original)
        .await
        .unwrap_err();

    assert!(res.is_not_found());

    Ok(())
}

use flake_id::FlakeIdGenerator;
use models::media::MediaVariant;
use result::Result;

use crate::{
    repos::media::MediaRepository,
    tests::contracts::media_repo::{prepare_media, prepare_media_file},
    types::{UpdateResult, patch::MediaFilePatch},
};

pub async fn update_existing_file<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);
    let thumb = prepare_media_file(&flake, &media, MediaVariant::Thumbnail);

    repo.insert(&media).await?;

    repo.insert_file(&original).await?;
    repo.insert_file(&thumb).await?;

    repo.update_file(&original.id, MediaFilePatch::new().duration_ms(Some(50)))
        .await?;

    let file = repo.get_variant(&media.id, MediaVariant::Original).await?;

    assert_eq!(file.duration_ms, Some(50));

    Ok(())
}

pub async fn return_not_found_when_updating_non_existent_file<R: MediaRepository>(
    repo: R,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let res = repo
        .update_file(
            &flake.get_id_as(),
            MediaFilePatch::new().duration_ms(Some(50)),
        )
        .await?;

    assert!(matches!(res, UpdateResult::NotFound));

    Ok(())
}

pub async fn delete_existing<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);

    repo.insert(&media).await?;
    repo.insert_file(&original).await?;

    repo.delete(&media.id).await?;

    assert!(repo.get_by_id(&media.id).await.unwrap_err().is_not_found());
    assert!(
        repo.get_variant(&media.id, MediaVariant::Original)
            .await
            .unwrap_err()
            .is_not_found()
    );

    Ok(())
}

pub async fn return_no_changes_when_deleting_non_existent<R: MediaRepository>(
    repo: R,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let res = repo.delete(&flake.get_id_as()).await?;
    assert!(res.no_changes());

    Ok(())
}

pub async fn delete_existing_file<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let media = prepare_media(&flake);

    let original = prepare_media_file(&flake, &media, MediaVariant::Original);

    repo.insert(&media).await?;
    repo.insert_file(&original).await?;

    repo.delete_file(&original.id).await?;

    assert!(
        repo.get_variant(&media.id, MediaVariant::Original)
            .await
            .unwrap_err()
            .is_not_found()
    );

    Ok(())
}

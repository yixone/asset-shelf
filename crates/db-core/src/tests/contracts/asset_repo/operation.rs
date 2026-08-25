use result::create_error;

use super::*;

/// Tests [`Asset`] creation via Operation when inserting an [`Asset`] with relations
pub async fn insert_with_related_and_commit<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let (m, a, af) = prepare_asset(&flake, "foo");

    let mut op = repo.create_op().await?;

    op.insert_media(&m).await?;
    op.insert_asset(&a).await?;
    op.insert_features(&af).await?;

    op.commit().await?;

    // Checks that the inserted asset can be retrieved by id
    let asset = repo.get_by_id(a.id).await?;
    assert_eq!(asset.inner, a);

    Ok(())
}

/// Tests [`Asset`] creation and rollback after insertion
pub async fn insert_with_related_and_rollback<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let (m, a, af) = prepare_asset(&flake, "foo");

    let mut op = repo.create_op().await?;

    op.insert_media(&m).await?;
    op.insert_asset(&a).await?;
    op.insert_features(&af).await?;

    op.rollback().await?;

    // Checks that the asset was not inserted into the database after a rollback
    let err = repo.get_by_id(a.id).await.unwrap_err();
    assert!(err.is_not_found());

    Ok(())
}

/// Checks that the [`Asset`] will not be created by an atomic operation
/// if an error occurs before commit
pub async fn rollback_creation_after_error<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let (m, a, af) = prepare_asset(&flake, "foo");

    let _: Result<()> = {
        let mut op = repo.create_op().await?;

        op.insert_media(&m).await?;
        op.insert_asset(&a).await?;
        op.insert_features(&af).await?;

        Err(create_error!(NotFound))
    };

    let err = repo.get_by_id(a.id).await.unwrap_err();
    assert!(err.is_not_found());

    Ok(())
}

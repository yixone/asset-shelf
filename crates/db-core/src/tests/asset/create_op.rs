use result::{ErrorKind, create_error};

use super::*;

pub async fn commit<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let (m, a, af) = prepare_asset(&flake, "foo");

    let mut op = repo.create_op().await?;

    op.insert_media(&m).await?;
    op.insert_asset(&a).await?;
    op.insert_features(&af).await?;

    op.commit().await?;

    let asset = repo.get_by_id(a.id).await?;
    assert_eq!(asset.inner, a);

    Ok(())
}

pub async fn rollback<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let (m, a, af) = prepare_asset(&flake, "foo");

    let mut op = repo.create_op().await?;

    op.insert_media(&m).await?;
    op.insert_asset(&a).await?;
    op.insert_features(&af).await?;

    op.rollback().await?;

    let err = repo.get_by_id(a.id).await.expect_err(
        "After a rollback in `create_op`, the asset should not be added to the database",
    );

    assert!(matches!(err.kind(), ErrorKind::NotFound));

    Ok(())
}

pub async fn rollback_on_error<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let (m, a, af) = prepare_asset(&flake, "foo");

    let _: Result<()> = {
        let mut op = repo.create_op().await?;

        op.insert_media(&m).await?;
        op.insert_asset(&a).await?;
        op.insert_features(&af).await?;

        Err(create_error!(NotFound))
    };

    let err = repo.get_by_id(a.id).await.expect_err(
        "After a rollback in `create_op`, the asset should not be added to the database",
    );

    assert!(matches!(err.kind(), ErrorKind::NotFound));

    Ok(())
}

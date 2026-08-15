use models::types::AssetsOrdering;

use crate::types::Pagination;

use super::*;

/// Tests the return of an empty list of assets
pub async fn empty<R: AssetRepository>(repo: R) -> Result<()> {
    let list = repo
        .list(Pagination::new(50, 0), AssetsOrdering::Newest)
        .await?;

    assert!(
        list.is_empty(),
        "If there are no records, `list` should return an empty array"
    );

    Ok(())
}

pub async fn ordered<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let first = insert_asset(&repo, "foo", &flake).await?;
    let second = insert_asset(&repo, "bar", &flake).await?;

    {
        let list = repo
            .list(Pagination::new(50, 0), AssetsOrdering::Newest)
            .await?;

        assert_eq!(
            list[0].id(),
            second.id,
            "The last inserted asset should be returned first"
        );

        assert_eq!(
            list[1].id(),
            first.id,
            "The second item returned should be the second-to-last asset inserted"
        );
    }

    {
        let list = repo
            .list(Pagination::new(50, 0), AssetsOrdering::Oldest)
            .await?;

        assert_eq!(
            list[0].id(),
            first.id,
            "The first inserted asset should be returned first"
        );

        assert_eq!(
            list[1].id(),
            second.id,
            "The second item returned should be the second asset inserted"
        );
    }

    Ok(())
}

pub async fn with_pagination<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let first = insert_asset(&repo, "foo", &flake).await?;
    let second = insert_asset(&repo, "bar", &flake).await?;

    {
        let list = repo
            .list(Pagination::new(1, 0), AssetsOrdering::Newest)
            .await?;

        assert_eq!(
            list.len(),
            1,
            "With a pagination limit of `1`, the repository must return a single asset"
        );

        assert_eq!(
            list[0].id(),
            second.id,
            "The last inserted asset should be returned first"
        );
    }
    {
        let list = repo
            .list(Pagination::new(1, 1), AssetsOrdering::Newest)
            .await?;

        assert_eq!(
            list.len(),
            1,
            "With a pagination limit of `1`, the repository must return a single asset"
        );

        assert_eq!(
            list[0].id(),
            first.id,
            "The second item returned should be the second-to-last asset inserted"
        );
    }

    Ok(())
}

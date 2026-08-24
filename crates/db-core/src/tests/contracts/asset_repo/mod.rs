mod mutation;
mod operation;
mod processing;
mod retrieval;
mod similarity;

use flake_id::FlakeIdGenerator;
use mimetype::MimeKind;
use models::{
    assets::{Asset, AssetFeatures},
    media::{Media, MediaVariant},
};
use result::Result;

use crate::repos::asset::AssetRepository;

/// Tests all [`mutation`] contracts for the given [`AssetRepository`]
pub async fn test_asset_mutation<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: AssetRepository,
{
    // Testing asset updates
    mutation::update_existing(repo().await).await?;
    mutation::return_no_changes_when_updating_non_existent(repo().await).await?;

    // Tests asset feature updates
    mutation::update_existing_features(repo().await).await?;

    // Tests asset deletion
    mutation::delete_existing(repo().await).await?;
    mutation::return_no_changes_when_deleting_non_existent(repo().await).await?;

    Ok(())
}

/// Tests all [`operation`] contracts for the given [`AssetRepository`]
pub async fn test_asset_operation<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: AssetRepository,
{
    // Tests the atomic insertion of an asset with all related models
    operation::insert_with_related_and_commit(repo().await).await?;
    operation::insert_with_related_and_rollback(repo().await).await?;

    // Tests atomic insert rollback on error
    operation::rollback_creation_after_error(repo().await).await?;

    Ok(())
}

/// Tests all [`processing`] contracts for the given [`AssetRepository`]
pub async fn test_asset_processing<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: AssetRepository,
{
    // Tests receiving assets for processing
    processing::get_for_processing(repo().await).await?;

    Ok(())
}

/// Tests all [`retrieval`] contracts for the given [`AssetRepository`]
pub async fn test_asset_retrieval<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: AssetRepository,
{
    // Tests retrieving assets by ID
    retrieval::get_an_existing_by_id(repo().await).await?;
    retrieval::return_not_found_when_getting_a_non_existent_asset(repo().await).await?;

    // Testing the retrieval of deleted assets
    retrieval::get_deleted_list(repo().await).await?;

    // Tests getting a list of assets
    retrieval::list_empty(repo().await).await?;
    retrieval::list_ordered(repo().await).await?;
    retrieval::list_with_pagination(repo().await).await?;

    // Tests asset counting
    retrieval::count_assets(repo().await).await?;

    Ok(())
}

/// Tests all [`similarity`] contracts for the given [`AssetRepository`]
pub async fn test_asset_similarity<F, R>(repo: F) -> Result<()>
where
    F: AsyncFn() -> R,
    R: AssetRepository,
{
    // Tests candidate return for similar asset searches
    similarity::return_candidates_for_similar_search(repo().await).await?;

    // Tests the return of a list of assets based on the similar search operation
    similarity::list_from_similar_search_results(repo().await).await?;

    Ok(())
}

/// Creates a test [`Asset`] and inserts it using the test [`AssetRepository`]
pub(crate) async fn insert_asset<R: AssetRepository>(
    repo: &R,
    title: &str,
    flake: &FlakeIdGenerator,
) -> Result<Asset> {
    let (media, asset, asset_features) = prepare_asset(flake, title);

    insert_full_asset((&media, &asset, &asset_features), repo).await?;

    Ok(asset)
}

/// Creates a test [`Asset`] with all associated models
pub(crate) fn prepare_asset(
    flake: &FlakeIdGenerator,
    title: &str,
) -> (Media, Asset, AssetFeatures) {
    let media = Media::new(flake.get_id_as());
    let asset = Asset::new(
        flake.get_id_as(),
        media.id.clone(),
        MimeKind::Image,
        Some(title.into()),
        None,
        None,
    );
    let features = AssetFeatures::new(asset.id);

    (media, asset, features)
}

pub(crate) async fn insert_full_asset<R: AssetRepository>(
    (m, a, af): (&Media, &Asset, &AssetFeatures),
    repo: &R,
) -> Result<()> {
    let mut op = repo.create_op().await?;

    op.insert_media(m).await?;
    op.insert_asset(a).await?;
    op.insert_features(af).await?;

    op.commit().await
}

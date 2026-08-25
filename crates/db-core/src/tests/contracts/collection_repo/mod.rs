use flake_id::FlakeIdGenerator;
use models::collections::Collection;
use result::Result;

use crate::repos::{asset::AssetRepository, collection::CollectionRepository};

mod relations;
mod retrieval;

/// Tests all [`relations`] contracts for the given [`CollectionRepository`]
pub async fn test_collection_relations<F, AR, CR>(repo: F) -> Result<()>
where
    F: AsyncFn() -> (AR, CR),
    AR: AssetRepository,
    CR: CollectionRepository,
{
    // Tests adding an asset to a collection
    {
        let (ar, cr) = repo().await;
        relations::add_asset_to_collection(ar, cr).await?;
    }

    // Tests deleting an asset from a collection
    {
        let (ar, cr) = repo().await;
        relations::remove_asset_from_collection(ar, cr).await?;
    }
    {
        // Separately tests rejection of asset deletion attempts using a rel_id unrelated to the passed table
        // Eliminates a exploit that allows an asset to be deleted from any collection using only the rel_id
        let (ar, cr) = repo().await;
        relations::do_not_remove_asset_for_unrelated_collection(ar, cr).await?;
    }
    Ok(())
}

/// Tests all [`retrieval`] contracts for the given [`CollectionRepository`]
pub async fn test_collection_retrieval<F, AR, CR>(repo: F) -> Result<()>
where
    F: AsyncFn() -> (AR, CR),
    AR: AssetRepository,
    CR: CollectionRepository,
{
    // Tests getting a collection with additions
    {
        let (ar, cr) = repo().await;
        retrieval::get_collection_with_additions(ar, cr).await?;
    }

    // Tests getting a list of collection assets
    {
        let (ar, cr) = repo().await;
        retrieval::get_collection_items(ar, cr).await?;
    }

    Ok(())
}

/// Prepares [`Collection`] for testing
pub(crate) fn prepare_collection(flake: &FlakeIdGenerator, name: &str) -> Collection {
    Collection::new(flake.get_id_as(), name.to_string(), None)
}

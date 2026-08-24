use flake_id::FlakeIdGenerator;
use models::collections::Collection;
use result::Result;

use crate::repos::{asset::AssetRepository, collection::CollectionRepository};

mod relations;
mod retrieval;

/// Tests all [`retrieval`] contracts for the given [`CollectionRepository`]
pub async fn test_collection_retrieval<F, AR, CR>(repo: F) -> Result<()>
where
    F: AsyncFn() -> (AR, CR),
    AR: AssetRepository,
    CR: CollectionRepository,
{
    // Tests getting media variant
    let (ar, cr) = repo().await;
    retrieval::get_collection_with_additions(ar, cr).await?;

    Ok(())
}

/// Prepares [`Collection`] for testing
pub(crate) fn prepare_collection(flake: &FlakeIdGenerator, name: &str) -> Collection {
    Collection::new(flake.get_id_as(), name.to_string(), None)
}

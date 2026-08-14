//! Tests for the `Asset` domain CRUD operations
//!
//! ### Get By Id
//! - [`get_by_id::existing`] - Tests retrieving an existing asset by ID
//! - [`get_by_id::throw_error_on_missing`] - Tests whether an error is returned when retrieving a non-existent asset by id
//! - [`get_by_id::with_valid_relations`] - Tests the return of the view model with the correct relations
//!
//! ### Get List
//! - [`list::empty`] - Tests the return of an empty list of assets
//! - [`list::newest`] - Tests the return of a list of the newest assets
//! - [`list::oldest`] - Tests the return of a list of the oldest assets
//! - [`list::with_pagination`] - Tests the return of a paginated list of assets
//! - [`list::deleted`] - Tests the return of a list of deleted assets
//! - [`list::with_valid_relations`] - Tests the return of a list of asset view models with the correct relations
//!
//! ### Update
//! - [`update::existing`] - Tests the update of an existing asset
//! - [`update::return_not_found_on_missing`] - Tests the return of [`UpdateResult::NotFound`] for a non-existent asset
//!
//! ### Delete
//! - [`delete::existing`] - Tests the deletion of an existing asset
//! - [`delete::with_related`] - Tests the deletion of an asset along with all associated models
//! - [`delete::return_not_found_on_missing`] - Tests the return of [`DeleteResult::NotFound`] for a non-existent asset
//!
//! ### Get For Processing
//! Tests the asset return behavior for processing under specific conditions
//! - [`get_for_processing::on_pending`]
//! - [`get_for_processing::on_processing_fresh`]
//! - [`get_for_processing::on_processing_stale`]
//! - [`get_for_processing::on_failed_fresh`]
//! - [`get_for_processing::on_failed_stale`]
//! - [`get_for_processing::on_ready_complete`]
//! - [`get_for_processing::on_ready_incomplete`]
//! - [`get_for_processing::with_valid_relations`]
//!
//! ### Get Similar Candidates
//! - [`get_similar_candidates::return_valid_canditates`] - Tests the return of correct candidates for similar asset search
//! - [`get_similar_candidates::without_deleted`] - Tests for the absence of assets marked as deleted among the candidates
//! - [`get_similar_candidates::on_empty_candidates_list`] - Tests behavior with an empty list of candidates
//!
//! ### Get From Similar
//! - [`get_from_similar::with_valid_ordering`] - Tests the return of the list in the correct order
//! - [`get_from_similar::with_valid_relations`]
//!
//! ### Create Asset Op:
//! - [`create_op::commit`] - Tests changes recording during a commit
//! - [`create_op::rollback`] - Tests the rollback of changes
//! - [`create_op::rollback_on_error`] - Tests the rollback of changes upon an error

use flake_id::FlakeIdGenerator;
use models::types::AssetsOrdering;
use result::{ErrorKind, Result};

use crate::{repos::asset::AssetRepository, types::Pagination};

use helpers::*;

/// Testing asset retrieval by ID
pub mod get_by_id {
    use super::*;

    /// Tests that the existing asset is returned by ID
    pub async fn existing<R: AssetRepository>(repo: &R) -> Result<()> {
        // Initializes the FlakeId generator
        let flake = FlakeIdGenerator::new(0);

        // Creates and inserts the first asset
        let asset = {
            let (media, asset, asset_features) = helpers::prepare_asset(&flake, "foo");
            insert_full_asset((&media, &asset, &asset_features), repo).await?;
            asset
        };

        // Retrieves the asset using the ID of the first asset
        let fetched_asset = repo.get_by_id(asset.id).await?;

        assert_eq!(
            fetched_asset.id(),
            asset.id,
            "The identifier of the inserted asset and the received asset must match"
        );

        Ok(())
    }

    /// Tests that an [`ErrorKind::NotFound`] error is returned when attempting to retrieve a non-existent asset
    pub async fn throw_error_on_missing<R: AssetRepository>(repo: &R) -> Result<()> {
        // Initializes the FlakeId generator
        let flake = FlakeIdGenerator::new(0);

        let err = repo
            .get_by_id(flake.get_id_as())
            .await
            .expect_err("A `get_by_id` request for a non-existent asset should return an error");

        assert!(
            matches!(err.kind(), ErrorKind::NotFound),
            "A `NotFound` error should be returned for a `get_by_id` on a non-existent asset"
        );

        Ok(())
    }
}

/// Testing assets list retrieval
pub mod list {
    use super::*;

    /// Tests the return of an empty list of assets
    pub async fn empty<R: AssetRepository>(repo: &R) -> Result<()> {
        // Retrieves a paginated list of assets
        let list = repo
            .list(Pagination::new(50, 0), AssetsOrdering::Newest)
            .await?;

        assert!(
            list.is_empty(),
            "If there are no records, `list` should return an empty array"
        );

        Ok(())
    }

    /// Tests the return of a list of assets in ascending order
    pub async fn newest<R: AssetRepository>(repo: &R) -> Result<()> {
        // Initializes the FlakeId generator
        let flake = FlakeIdGenerator::new(0);

        // Creates and inserts the first asset
        let first = {
            let (media, asset, asset_features) = prepare_asset(&flake, "foo");
            insert_full_asset((&media, &asset, &asset_features), repo).await?;
            asset
        };

        // Creates and inserts the second asset
        let second = {
            let (media, asset, asset_features) = prepare_asset(&flake, "bar");
            insert_full_asset((&media, &asset, &asset_features), repo).await?;
            asset
        };

        // Retrieves a paginated list of assets
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

        Ok(())
    }
}

mod helpers {
    use flake_id::FlakeIdGenerator;
    use mimetype::MimeKind;
    use models::{
        assets::{Asset, AssetFeatures},
        media::Media,
    };
    use result::Result;

    use crate::repos::asset::AssetRepository;

    pub fn prepare_asset(flake: &FlakeIdGenerator, title: &str) -> (Media, Asset, AssetFeatures) {
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

    pub async fn insert_full_asset<R: AssetRepository>(
        (m, a, af): (&Media, &Asset, &AssetFeatures),
        repo: &R,
    ) -> Result<()> {
        let mut op = repo.create_op().await?;

        op.insert_media(m).await?;
        op.insert_asset(a).await?;
        op.insert_features(af).await?;

        op.commit().await
    }
}

use std::sync::Arc;

use db_core::tests::asset::{get_by_id, list};
use db_sqlite::{SqliteDatabase, repos::asset::SqliteAssetRepository};

async fn prepare_asset_repo() -> SqliteAssetRepository {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    SqliteAssetRepository { db }
}

/// Tests retrieving an existing asset by ID
#[tokio::test]
async fn get_by_id() {
    let repo = prepare_asset_repo().await;

    get_by_id::existing(&repo).await.unwrap();
}

/// Tests that an NotFound error is returned when attempting to retrieve a non-existent asset
#[tokio::test]
async fn get_by_id_and_throw_not_found() {
    let repo = prepare_asset_repo().await;

    get_by_id::throw_error_on_missing(&repo).await.unwrap();
}

/// Inserts multiple assets and tests retrieving a paginated list
#[tokio::test]
async fn list_newest() {
    let repo = prepare_asset_repo().await;

    list::newest(&repo).await.unwrap();
}

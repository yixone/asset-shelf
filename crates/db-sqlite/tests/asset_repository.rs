use std::sync::Arc;

use db_core::tests::contracts;
use db_sqlite::{SqliteDatabase, repos::asset::SqliteAssetRepository};

async fn repo() -> SqliteAssetRepository {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    SqliteAssetRepository { db }
}

#[tokio::test]
async fn asset_mutation() {
    contracts::asset_repository::test_asset_mutation(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_operations() {
    contracts::asset_repository::test_asset_operation(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_processing() {
    contracts::asset_repository::test_asset_processing(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_retrieval() {
    contracts::asset_repository::test_asset_retrieval(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_similarity() {
    contracts::asset_repository::test_asset_similarity(repo)
        .await
        .unwrap();
}

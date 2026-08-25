use std::sync::Arc;

use db_core::tests::contracts;
use db_sqlite::{
    SqliteDatabase,
    repos::{asset::SqliteAssetRepository, collection::SqliteCollectionRepository},
};

async fn repo_with_assets() -> (SqliteAssetRepository, SqliteCollectionRepository) {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    (
        SqliteAssetRepository { db: db.clone() },
        SqliteCollectionRepository { db },
    )
}

#[tokio::test]
async fn collection_retrieval() {
    contracts::collection_repo::test_collection_retrieval(repo_with_assets)
        .await
        .unwrap();
}

#[tokio::test]
async fn collection_relations() {
    contracts::collection_repo::test_collection_relations(repo_with_assets)
        .await
        .unwrap();
}

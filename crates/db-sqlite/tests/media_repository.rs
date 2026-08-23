use std::sync::Arc;

use db_core::tests::contracts;
use db_sqlite::{
    SqliteDatabase,
    repos::{asset::SqliteAssetRepository, media::SqliteMediaRepository},
};

async fn repo() -> SqliteMediaRepository {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    SqliteMediaRepository { db }
}

async fn repo_with_assets() -> (SqliteMediaRepository, SqliteAssetRepository) {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    (
        SqliteMediaRepository { db: db.clone() },
        SqliteAssetRepository { db: db.clone() },
    )
}

#[tokio::test]
async fn media_insertion() {
    contracts::media_repo::test_media_insertion(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn media_mutation() {
    contracts::media_repo::test_media_mutation(repo)
        .await
        .unwrap();
}

#[tokio::test]
async fn media_relations() {
    contracts::media_repo::test_media_relations(repo_with_assets)
        .await
        .unwrap();
}

#[tokio::test]
async fn media_retrieval() {
    contracts::media_repo::test_media_retrieval(repo)
        .await
        .unwrap();
}

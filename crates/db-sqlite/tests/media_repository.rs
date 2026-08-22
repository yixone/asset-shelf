use std::sync::Arc;

use db_core::tests::contracts;
use db_sqlite::{SqliteDatabase, repos::media::SqliteMediaRepository};

async fn repo() -> SqliteMediaRepository {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    SqliteMediaRepository { db }
}

#[tokio::test]
async fn media_insertion() {
    contracts::media_repository::test_media_insertion(repo)
        .await
        .unwrap();
}

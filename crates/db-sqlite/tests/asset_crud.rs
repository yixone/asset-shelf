use std::sync::Arc;

use db_core::tests::asset::{delete, get_by_id, get_deleted, get_for_processing, list, update};
use db_sqlite::{SqliteDatabase, repos::asset::SqliteAssetRepository};

async fn repo() -> SqliteAssetRepository {
    let db = Arc::new(SqliteDatabase::open_in_mem().await.unwrap());
    db.migrate().await.unwrap();
    SqliteAssetRepository { db }
}

#[tokio::test]
async fn asset_get_by_id() {
    get_by_id::get_existing(repo().await).await.unwrap();

    get_by_id::throw_error_on_missing(repo().await)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_list() {
    list::empty(repo().await).await.unwrap();

    list::ordered(repo().await).await.unwrap();

    list::with_pagination(repo().await).await.unwrap();
}

#[tokio::test]
async fn asset_get_deleted() {
    get_deleted::get_deleted_list(repo().await).await.unwrap();
}

#[tokio::test]
async fn asset_update() {
    update::update_existing(repo().await).await.unwrap();

    update::return_not_found_on_missing(repo().await)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_delete() {
    delete::delete_existing(repo().await).await.unwrap();

    delete::return_no_changes_on_missing(repo().await)
        .await
        .unwrap();
}

#[tokio::test]
async fn asset_get_for_processing() {
    get_for_processing::with_states(repo().await).await.unwrap();
}

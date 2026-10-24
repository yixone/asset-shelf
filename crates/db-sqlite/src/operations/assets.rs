use db_api::{DatabaseError, result::ToDatabaseError};
use models::id::AssetId;
use sqlx::{QueryBuilder, SqliteConnection};

use crate::schema::{DbAsset, DbAssetFeatures, DbAssetMeta};

pub async fn insert_asset(
    asset: &DbAsset,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
            INSERT INTO assets (
                id,
                parent_id, duplicate_of,
                file_group_id,
                checksum, original_filename, original_mime_kind,
                state, state_failure,
                is_offline,
                created_at, updated_at, deleted_at
            ) VALUES (
                ?,
                ?, ?,
                ?,
                ?, ?, ?,
                ?, ?,
                ?,
                ?, ?, ?
            )
            ",
    )
    .bind(asset.id)
    .bind(asset.parent_id)
    .bind(asset.duplicate_of)
    .bind(&asset.file_group_id)
    .bind(&asset.checksum)
    .bind(&asset.original_filename)
    .bind(asset.original_mime_kind)
    .bind(&asset.state)
    .bind(asset.state_failure)
    .bind(asset.is_offline)
    .bind(asset.created_at)
    .bind(asset.updated_at)
    .bind(asset.deleted_at)
    .execute(conn)
    .await
    .to_db_error()?;
    Ok(())
}

pub async fn insert_asset_features(
    features: &DbAssetFeatures,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
            INSERT INTO assets_features (
                id,
                accent_color,
                width, height,
                p_hash, a_hash
            ) VALUES (
                ?,
                ?, 
                ?, ?,
                ?, ?
            )
            ",
    )
    .bind(features.id)
    .bind(features.accent_color)
    .bind(features.width)
    .bind(features.height)
    .bind(features.p_hash)
    .bind(features.a_hash)
    .execute(conn)
    .await
    .to_db_error()?;
    Ok(())
}

pub async fn insert_asset_meta(
    meta: &DbAssetMeta,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
            INSERT INTO assets_meta (
                id,
                name, caption, source_url
            ) VALUES (
                ?,
                ?, ?, ?
            )
            ",
    )
    .bind(meta.id)
    .bind(&meta.name)
    .bind(&meta.caption)
    .bind(&meta.source_url)
    .execute(conn)
    .await
    .to_db_error()?;
    Ok(())
}

pub async fn load_asset_features_many(
    ids: &[AssetId],
    conn: &mut SqliteConnection,
) -> Result<Vec<DbAssetFeatures>, DatabaseError> {
    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM assets_features
            WHERE id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });
    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_asset_features(
    id: AssetId,
    conn: &mut SqliteConnection,
) -> Result<Option<DbAssetFeatures>, DatabaseError> {
    load_asset_features_many(&[id], conn)
        .await
        .map(|a| a.into_iter().next())
}

pub async fn load_asset_meta_many(
    ids: &[AssetId],
    conn: &mut SqliteConnection,
) -> Result<Vec<DbAssetMeta>, DatabaseError> {
    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM assets_meta
            WHERE id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });
    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_asset_meta(
    id: AssetId,
    conn: &mut SqliteConnection,
) -> Result<Option<DbAssetMeta>, DatabaseError> {
    load_asset_meta_many(&[id], conn)
        .await
        .map(|a| a.into_iter().next())
}

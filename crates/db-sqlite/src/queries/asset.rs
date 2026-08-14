use db_core::types::InsertResult;
use models::{
    assets::{Asset, AssetFeatures},
    types::AssetId,
};
use result::{Result, error::ResultExt};
use sqlx::{Executor, QueryBuilder, Sqlite};

pub async fn insert_asset<'a, E>(asset: &Asset, exec: E) -> Result<InsertResult>
where
    E: Executor<'a, Database = Sqlite>,
{
    let res = sqlx::query(
        "
            INSERT INTO assets (
                id, state,
                media_id, media_type,
                created_at, updated_at, deleted_at,
                title, caption, source_url
            )
            VALUES (
                ?, ?,
                ?, ?,
                ?, ?, ?,
                ?, ?, ?
            )
            ",
    )
    .bind(asset.id)
    .bind(asset.state)
    .bind(&asset.media_id)
    .bind(asset.media_type)
    .bind(asset.created_at)
    .bind(asset.updated_at)
    .bind(asset.deleted_at)
    .bind(&asset.title)
    .bind(&asset.caption)
    .bind(&asset.source_url)
    .execute(exec)
    .await
    .to_app_err()?;

    Ok(res.into())
}

pub async fn insert_asset_features<'a, E>(features: &AssetFeatures, exec: E) -> Result<InsertResult>
where
    E: Executor<'a, Database = Sqlite>,
{
    let res = sqlx::query(
        "
            INSERT INTO asset_features (
                asset_id, p_hash, a_hash,
                width, height, accent_color
            )
            VALUES (
                ?, ?, ?,
                ?, ?, ?
            )
            ",
    )
    .bind(features.asset_id)
    .bind(features.p_hash)
    .bind(features.a_hash)
    .bind(features.width)
    .bind(features.height)
    .bind(features.accent_color)
    .execute(exec)
    .await
    .to_app_err()?;

    Ok(res.into())
}

pub async fn get_assets_features<'a, E>(ids: &[AssetId], exec: E) -> Result<Vec<AssetFeatures>>
where
    E: Executor<'a, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM asset_features
            WHERE asset_id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });

    qb.build_query_as().fetch_all(exec).await.to_app_err()
}

pub async fn get_assets<'a, E>(ids: &[AssetId], exec: E) -> Result<Vec<Asset>>
where
    E: Executor<'a, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM assets
            WHERE id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });

    qb.build_query_as().fetch_all(exec).await.to_app_err()
}

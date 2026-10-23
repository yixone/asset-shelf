use std::collections::HashMap;

use chrono::Utc;
use db_api::{
    DatabaseError,
    ports::{AssetDatabase, DeleteFn, UpdateFn},
    result::ToDatabaseError,
    types::{AssetFull, CursorPagination, DeletedVisibility, SortOrder},
};
use models::{
    EntityError,
    entities::{Asset, FileGroup},
    id::AssetId,
    types::{AssetSortBy, Sha256Checksum},
};
use sqlx::{QueryBuilder, SqliteConnection};

use crate::{
    driver::SqliteDatabase,
    impls::files,
    schema::{DbAsset, DbAssetFeatures, DbAssetMeta, DbFileGroup},
};

pub struct SqliteAssetsRepository {
    db: SqliteDatabase,
}

#[async_trait::async_trait]
impl AssetDatabase for SqliteAssetsRepository {
    async fn insert(&self, asset: &Asset, file_group: &FileGroup) -> Result<(), DatabaseError> {
        let mut tx = self.db.begin().await?;

        let (group, files) = DbFileGroup::from_domain(file_group);
        files::insert_file_group(&group, &mut *tx).await?;
        for f in files {
            files::insert_file(&f, &mut *tx).await?;
        }

        let (asset, features, meta) = DbAsset::from_domain(asset);
        insert_asset(&asset, &mut tx).await?;
        insert_features(&features, &mut tx).await?;
        insert_meta(&meta, &mut tx).await?;

        tx.commit().await.to_db_error()?;
        Ok(())
    }

    async fn get_many_by_ids(&self, ids: &[AssetId]) -> Result<Vec<AssetFull>, DatabaseError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Uses a transaction to obtain a consistent collectable domain
        let mut tx = self.db.begin().await?;

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM assets
            WHERE id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });
        let assets = qb
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .to_db_error()?;

        let ret = hydrate_assets_full(assets, &mut tx).await?;

        tx.commit().await.to_db_error()?;

        Ok(ret)
    }

    async fn get_by_checksum(
        &self,
        checksum: &Sha256Checksum,
    ) -> Result<Option<Asset>, DatabaseError> {
        // Uses a transaction to obtain a consistent collectable domain
        let mut tx = self.db.begin().await?;

        let Some(asset) = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE checksum = ?
            ",
        )
        .bind(checksum.to_vec())
        .fetch_optional(&mut *tx)
        .await
        .to_db_error()?
        else {
            return Ok(None);
        };

        let ret = hydrate_assets(vec![asset], &mut tx)
            .await?
            .into_iter()
            .next();

        tx.commit().await.to_db_error()?;

        Ok(ret)
    }

    async fn list(
        &self,
        pagination: CursorPagination<AssetId>,
        ordering: (AssetSortBy, SortOrder),
        deleted: DeletedVisibility,
    ) -> Result<Vec<AssetFull>, DatabaseError> {
        if pagination.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
                SELECT *
                FROM assets
                WHERE 1=1
                ",
        );

        let (order_by, ordering) = ordering;

        match deleted {
            DeletedVisibility::Show => (),
            DeletedVisibility::Hide => {
                qb.push(" AND deleted_at IS null ");
            }
            DeletedVisibility::DeletedOnly => {
                qb.push(" AND deleted_at IS NOT null ");
            }
        }

        if let Some(cursor) = pagination.cursor() {
            if matches!(ordering, SortOrder::Asc) {
                qb.push(" AND id > ");
            } else {
                qb.push(" AND id < ");
            }
            qb.push_bind(cursor);
        }

        qb.push(" ORDER BY ");

        match order_by {
            AssetSortBy::CreatedAt => qb.push("created_at"),
            AssetSortBy::DeletedAt => qb.push("deleted_at"),
        };
        match ordering {
            SortOrder::Asc => qb.push(" ASC "),
            SortOrder::Desc => qb.push(" DESC "),
        };

        qb.push(", id ");
        match ordering {
            SortOrder::Asc => qb.push(" ASC "),
            SortOrder::Desc => qb.push(" DESC "),
        };

        qb.push(" LIMIT ");
        qb.push_bind(pagination.limit() as i64);

        // Uses a transaction to obtain a consistent collectable domain
        let mut tx = self.db.begin().await?;

        let assets = qb
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .to_db_error()?;

        let ret = hydrate_assets_full(assets, &mut tx).await?;

        tx.commit().await.to_db_error()?;

        Ok(ret)
    }

    async fn get_random(&self) -> Result<Option<AssetFull>, DatabaseError> {
        // Uses a transaction to obtain a consistent collectable domain
        let mut tx = self.db.begin().await?;

        let Some(asset) = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE deleted_at IS null
            ORDER BY RANDOM()
            ",
        )
        .fetch_optional(&mut *tx)
        .await
        .to_db_error()?
        else {
            return Ok(None);
        };

        let ret = hydrate_assets_full(vec![asset], &mut tx)
            .await?
            .into_iter()
            .next();

        tx.commit().await.to_db_error()?;

        Ok(ret)
    }

    async fn count_total(&self, deleted: DeletedVisibility) -> Result<u64, DatabaseError> {
        let mut qb = QueryBuilder::new(
            "
            SELECT COUNT(id)
            FROM assets
            ",
        );

        match deleted {
            DeletedVisibility::Show => qb.push(""),
            DeletedVisibility::Hide => qb.push(" WHERE deleted_at IS null "),
            DeletedVisibility::DeletedOnly => qb.push(" WHERE deleted_at IS NOT null "),
        };

        let count = qb
            .build_query_scalar::<i64>()
            .fetch_one(self.db.pool())
            .await
            .to_db_error()?;

        Ok(count as u64)
    }

    async fn update_fn_boxed(
        &self,
        id: AssetId,
        f: Box<UpdateFn<Asset>>,
    ) -> Result<Asset, DatabaseError> {
        let mut tx = self.db.begin().await?;

        let asset = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .to_db_error()?
        .ok_or(DatabaseError::Entity(EntityError::NotFound))?;

        let mut domain = hydrate_assets(vec![asset], &mut tx)
            .await?
            .into_iter()
            .next()
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;

        f(&mut domain)?;

        let updated_at = Utc::now();
        let (a, f, m) = DbAsset::from_domain(&domain);

        let updated_a: DbAsset = sqlx::query_as(
            "
            UPDATE assets
            SET 
                updated_at = ?, parent_id = ?,
                duplicate_of = ?, state = ?,
                state_failure = ?, is_offline = ?,
                deleted_at = ?
            WHERE id = ?
            RETURNING *
            ",
        )
        .bind(updated_at)
        .bind(a.parent_id)
        .bind(a.duplicate_of)
        .bind(a.state)
        .bind(a.state_failure)
        .bind(a.is_offline)
        .bind(a.deleted_at)
        .bind(a.id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let updated_f: DbAssetFeatures = sqlx::query_as(
            "
            UPDATE assets_features
            SET 
                accent_color = ?,
                width = ?, height = ?,
                p_hash = ?, a_hash = ?
            WHERE id = ?
            RETURNING *
            ",
        )
        .bind(f.accent_color)
        .bind(f.width)
        .bind(f.height)
        .bind(f.p_hash)
        .bind(f.a_hash)
        .bind(f.id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let updated_m: DbAssetMeta = sqlx::query_as(
            "
            UPDATE assets_meta
            SET 
                name = ?, caption = ?, source_url = ?
            WHERE id = ?
            RETURNING *
            ",
        )
        .bind(m.name)
        .bind(m.caption)
        .bind(m.source_url)
        .bind(m.id)
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        let asset = DbAsset::into_domain((updated_a, updated_f, updated_m))?;

        tx.commit().await.to_db_error()?;

        Ok(asset)
    }

    async fn delete_fn_boxed(
        &self,
        id: AssetId,
        f: Box<DeleteFn<Asset>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.db.begin().await?;

        let asset = sqlx::query_as(
            "
            SELECT * FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .to_db_error()?
        .ok_or(DatabaseError::Entity(EntityError::NotFound))?;

        let domain = hydrate_assets(vec![asset], &mut tx)
            .await?
            .into_iter()
            .next()
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;

        f(&domain)?;

        sqlx::query(
            "
            DELETE FROM assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(())
    }
}

async fn insert_asset(asset: &DbAsset, conn: &mut SqliteConnection) -> Result<(), DatabaseError> {
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

async fn insert_features(
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

async fn get_features(
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

async fn get_meta(
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

async fn insert_meta(meta: &DbAssetMeta, conn: &mut SqliteConnection) -> Result<(), DatabaseError> {
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

async fn hydrate_assets(
    assets: Vec<DbAsset>,
    conn: &mut SqliteConnection,
) -> Result<Vec<Asset>, DatabaseError> {
    let ids = assets.iter().map(|a| a.id).collect::<Vec<_>>();

    let features = get_features(&ids, conn).await?;
    let meta = get_meta(&ids, conn).await?;

    let assets = unit_asset_domain(assets, features, meta)?;

    Ok(assets)
}

async fn hydrate_assets_full(
    assets: Vec<DbAsset>,
    conn: &mut SqliteConnection,
) -> Result<Vec<AssetFull>, DatabaseError> {
    let assets = hydrate_assets(assets, conn).await?;

    let file_groups_ids = assets
        .iter()
        .map(|a| a.file_group_id().clone())
        .collect::<Vec<_>>();

    let file_groups = files::get_file_groups_by_ids(&file_groups_ids, conn).await?;
    let files = files::get_files_by_groups_ids(&file_groups_ids, conn).await?;

    let files = files::unit_files_domain(file_groups, files)?;

    let ret = unit_assets_full(assets, files);

    Ok(ret)
}

/// Combines parts of asset records into an array of unit tuples
///
/// Ignores incomplete records (i.e., those missing at least one domain component)
fn unit_asset_domain(
    assets: Vec<DbAsset>,
    features: Vec<DbAssetFeatures>,
    meta: Vec<DbAssetMeta>,
) -> Result<Vec<Asset>, DatabaseError> {
    let mut features_idx = features
        .into_iter()
        .map(|f| (f.id, f))
        .collect::<HashMap<_, _>>();
    let mut meta_idx = meta
        .into_iter()
        .map(|m| (m.id, m))
        .collect::<HashMap<_, _>>();

    assets
        .into_iter()
        .filter_map(|a| {
            let features = features_idx.remove(&a.id)?;
            let meta = meta_idx.remove(&a.id)?;

            Some((a, features, meta))
        })
        .map(DbAsset::into_domain)
        .collect::<Result<Vec<_>, _>>()
}

fn unit_assets_full(assets: Vec<Asset>, files: Vec<FileGroup>) -> Vec<(Asset, FileGroup)> {
    let mut files_idx = files
        .into_iter()
        .map(|f| (f.id().clone(), f))
        .collect::<HashMap<_, _>>();

    assets
        .into_iter()
        .filter_map(|a| {
            let files = files_idx.remove(a.file_group_id())?;
            Some((a, files))
        })
        .collect::<Vec<_>>()
}

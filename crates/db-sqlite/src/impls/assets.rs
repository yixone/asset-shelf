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
use sqlx::QueryBuilder;

use crate::{
    coalesced,
    driver::SqliteDatabase,
    operations,
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
        operations::insert_file_group(&group, &mut tx).await?;
        for f in files {
            operations::insert_file(&f, &mut tx).await?;
        }

        let (asset, features, meta) = DbAsset::from_domain(asset);
        operations::insert_asset(&asset, &mut tx).await?;
        operations::insert_asset_features(&features, &mut tx).await?;
        operations::insert_asset_meta(&meta, &mut tx).await?;

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
            WHERE id IN (
            ",
        );

        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(id);
        }
        sep.push_unseparated(" )");

        let assets = qb
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .to_db_error()?;

        let ret = coalesced::hydrate_assets_full(assets, &mut tx).await?;

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

        let ret = coalesced::hydrate_asset(asset, &mut tx).await?;

        tx.commit().await.to_db_error()?;

        Ok(Some(ret))
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

        let ret = coalesced::hydrate_assets_full(assets, &mut tx).await?;

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

        let ret = coalesced::hydrate_asset_full(asset, &mut tx).await?;

        tx.commit().await.to_db_error()?;

        Ok(Some(ret))
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

        let mut domain = coalesced::hydrate_asset(asset, &mut tx).await?;

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

        let domain = coalesced::hydrate_asset(asset, &mut tx).await?;

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

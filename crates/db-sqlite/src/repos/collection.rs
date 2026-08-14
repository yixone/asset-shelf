use std::sync::Arc;

use chrono::Utc;
use db_core::{
    repos::collection::CollectionRepository,
    types::{DeleteResult, InsertResult, Pagination, UpdateResult, patch::CollectionPatch},
};
use models::{
    collections::{
        Collection, CollectionAsset,
        view::{CollectionItemView, CollectionView},
    },
    types::{
        AssetId, CollectionAssetId, CollectionAssetsOrdering, CollectionId, CollectionsOrdering,
    },
};
use result::{Result, error::ResultExt};
use sqlx::QueryBuilder;

use crate::{driver::SqliteDatabase, helpers::hydrate};

pub struct SqliteCollectionRepository {
    pub db: Arc<SqliteDatabase>,
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn insert(&self, collection: &Collection) -> Result<InsertResult> {
        let res = sqlx::query(
            "
            INSERT INTO collections (
                id,
                name, description,
                created_at
            )
            VALUES (
                ?,
                ?, ?,
                ?
            )
            ",
        )
        .bind(collection.id)
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(collection.created_at)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(res.into())
    }

    async fn update(
        &self,
        id: CollectionId,
        patch: CollectionPatch,
    ) -> Result<UpdateResult<CollectionView>> {
        let mut qb = QueryBuilder::new(
            "
            UPDATE collections
            SET
            ",
        );

        patch.apply_sql(&mut qb);

        qb.push(" WHERE id = ");
        qb.push_bind(id);

        qb.push(" RETURNING * ");

        let res = qb.build().execute(self.db.exec()).await.to_app_err()?;
        if res.rows_affected() == 0 {
            return Ok(UpdateResult::NotFound);
        }

        let asset = self.get_by_id(id).await?;
        Ok(UpdateResult::Updated(asset))
    }

    async fn delete(&self, id: CollectionId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM collections
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(res.into())
    }

    async fn get_items(
        &self,
        id: CollectionId,
        pagination: Pagination,
        order: CollectionAssetsOrdering,
    ) -> Result<Vec<CollectionItemView>> {
        if pagination.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT ca.* 
            FROM collections AS c
            JOIN collection_assets AS ca
                ON ca.collection_id = c.id
            JOIN assets AS a
                ON a.id = ca.asset_id
            WHERE a.deleted_at IS NULL
            ",
        );
        let mut query = qb.separated(" ");

        query.push("AND c.id =");
        query.push_bind(id);

        match order {
            CollectionAssetsOrdering::Latest => query.push("ORDER BY ca.added_at DESC"),
            CollectionAssetsOrdering::Oldest => query.push("ORDER BY ca.added_at ASC"),
        };
        pagination.apply_sql(&mut qb);

        let mut conn = self.db.acquire().await?;

        let items = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_collection_assets(items, &mut conn).await
    }

    async fn add_asset(
        &self,
        rel: CollectionAssetId,
        id: CollectionId,
        asset: AssetId,
    ) -> Result<CollectionAsset> {
        let ca = CollectionAsset {
            id: rel,
            asset_id: asset,
            collection_id: id,
            added_at: Utc::now(),
        };

        sqlx::query(
            "
            INSERT INTO collection_assets (
                id, collection_id, asset_id, added_at
            )
            VALUES (
                ?, ?, ?, ?
            )
            ",
        )
        .bind(ca.id)
        .bind(ca.collection_id)
        .bind(ca.asset_id)
        .bind(ca.added_at)
        .execute(self.db.exec())
        .await
        .to_app_err()?;

        Ok(ca)
    }

    async fn remove_asset(&self, id: CollectionId, rel: CollectionAssetId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM collection_assets
            WHERE id = ? AND collection_id = ?
            ",
        )
        .bind(rel)
        .bind(id)
        .execute(self.db.exec())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn get_by_ids(&self, ids: &[CollectionId]) -> Result<Vec<CollectionView>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT * FROM collections
            WHERE id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let mut conn = self.db.acquire().await?;

        let collections = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_collections(collections, &mut conn).await
    }

    async fn list(
        &self,
        pagination: Pagination,
        order: CollectionsOrdering,
    ) -> Result<Vec<CollectionView>> {
        if pagination.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT c.*
            FROM collections AS c
            LEFT JOIN collection_assets AS ca 
            ON ca.collection_id = c.id
            AND ca.added_at = (
                SELECT MAX(added_at)
                FROM collection_assets AS ca
                WHERE ca.collection_id = c.id
            )
            ",
        );
        let mut query = qb.separated(" ");

        match order {
            CollectionsOrdering::Latest => query.push("ORDER BY ca.added_at DESC"),
            CollectionsOrdering::Oldest => query.push("ORDER BY ca.added_at ASC"),
        };
        pagination.apply_sql(&mut qb);

        let mut conn = self.db.acquire().await?;

        let collections = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .to_app_err()?;

        hydrate::hydrate_collections(collections, &mut conn).await
    }
}

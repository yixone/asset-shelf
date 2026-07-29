use flake_id::str::FlakeIdStr;
use models::{
    entities::{Collection, CollectionAdditions, CollectionAsset},
    types::{AssetsOrdering, CollectionAssetId, CollectionId, MediaId},
};
use result::{Result, error::ResultExt};
use sqlx::{QueryBuilder, prelude::FromRow};

use crate::{
    ops::{CollectionAssetsOps, CollectionsOps},
    sqlite::SqliteExecutor,
    types::{DeleteResult, InsertResult, Pagination, UpdateResult, patches::CollectionPatch},
};

impl<T> CollectionsOps for T
where
    T: SqliteExecutor,
{
    async fn insert_collection(&mut self, c: &Collection) -> Result<InsertResult> {
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
        .bind(c.id)
        .bind(&c.name)
        .bind(&c.description)
        .bind(c.created_at)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn update_collection(
        &mut self,
        id: CollectionId,
        patch: CollectionPatch,
    ) -> Result<UpdateResult<Collection>> {
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

        let res = qb
            .build_query_as()
            .fetch_optional(self.executor())
            .await
            .to_app_err()?;
        Ok(res.into())
    }

    async fn delete_collection(&mut self, id: CollectionId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM collections
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn list_collections(&mut self, p: Pagination) -> Result<Vec<Collection>> {
        if p.limit() == 0 {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT c.* FROM collections AS c
            ",
        );
        p.apply_sql(&mut qb);

        qb.build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()
    }

    async fn get_collections_bulk(&mut self, ids: &[CollectionId]) -> Result<Vec<Collection>> {
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

        qb.build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()
    }

    async fn get_collections_additions_bulk(
        &mut self,
        ids: &[CollectionId],
    ) -> Result<Vec<CollectionAdditions>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut qb = QueryBuilder::new(
            "
            SELECT 
            c.id,
            (
                SELECT group_concat(media_id, ';')
                FROM (
                    SELECT a.media_id
                    FROM collection_assets AS ca
                    JOIN assets AS a 
                        ON a.id = ca.asset_id
                    WHERE ca.collection_id = c.id AND a.deleted_at IS NULL
                    ORDER BY ca.added_at DESC
                    LIMIT 3 OFFSET 0
                )
            ) AS thumbnails,
            (
                SELECT COUNT(a.id)
                FROM collection_assets AS ca
                LEFT JOIN assets AS a ON a.id = ca.asset_id
                WHERE a.deleted_at IS NULL AND ca.collection_id = c.id
            ) AS assets_count 
            FROM collections AS c
            WHERE c.id IN
            ",
        );
        qb.push_tuples(ids, |mut qb, id| {
            qb.push_bind(id);
        });

        let additions: Vec<CollectionAdditionsRow> = qb
            .build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()?;

        Ok(additions
            .into_iter()
            .map(CollectionAdditions::from)
            .collect())
    }
}

impl<T> CollectionAssetsOps for T
where
    T: SqliteExecutor,
{
    async fn insert_collection_asset(&mut self, ca: &CollectionAsset) -> Result<InsertResult> {
        let res = sqlx::query(
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
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn remove_collection_asset(&mut self, id: &CollectionAssetId) -> Result<DeleteResult> {
        let res = sqlx::query(
            "
            DELETE FROM collection_assets
            WHERE id = ?
            ",
        )
        .bind(id)
        .execute(self.executor())
        .await
        .to_app_err()?;
        Ok(res.into())
    }

    async fn get_collection_assets(
        &mut self,
        id: &CollectionId,
        p: Pagination,
        o: AssetsOrdering,
    ) -> Result<Vec<CollectionAsset>> {
        if p.limit() == 0 {
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

        match o {
            AssetsOrdering::Newest => query.push("ORDER BY ca.added_at DESC"),
            AssetsOrdering::Oldest => query.push("ORDER BY ca.added_at ASC"),
        };
        p.apply_sql(&mut qb);

        qb.build_query_as()
            .fetch_all(self.executor())
            .await
            .to_app_err()
    }
}

/// Raw database row for mapping in [`CollectionAdditions`]
#[derive(FromRow)]
pub struct CollectionAdditionsRow {
    pub id: CollectionId,
    /// The list of previews is fetched as a concatenated list of
    /// MediaIds and split back apart upon entering the domain
    pub thumbnails: String,
    pub assets_count: u32,
}

impl From<CollectionAdditionsRow> for CollectionAdditions {
    fn from(row: CollectionAdditionsRow) -> Self {
        let ids = row
            .thumbnails
            .split(';')
            .map(|v| MediaId(FlakeIdStr(v.to_string())))
            .collect::<Vec<_>>();

        CollectionAdditions {
            collection: row.id,
            thumbnails: ids,
            assets_count: row.assets_count,
        }
    }
}

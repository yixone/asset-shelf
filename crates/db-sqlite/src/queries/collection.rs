use models::{entities::CollectionAdditions, types::CollectionId};
use result::{Result, error::ResultExt};
use sqlx::{Executor, QueryBuilder, Sqlite};

use crate::helpers::rows::CollectionAdditionsRow;

pub async fn get_collections_additions<'a, E>(
    ids: &[CollectionId],
    exec: E,
) -> Result<Vec<CollectionAdditions>>
where
    E: Executor<'a, Database = Sqlite>,
{
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

    let additions: Vec<CollectionAdditionsRow> =
        qb.build_query_as().fetch_all(exec).await.to_app_err()?;

    Ok(additions
        .into_iter()
        .map(CollectionAdditions::from)
        .collect())
}

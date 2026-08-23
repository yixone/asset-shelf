use db_core::types::InsertResult;
use models::{
    media::{Media, MediaFile},
    types::MediaId,
};
use result::{Result, create_error, error::ResultExt};
use sqlx::{Executor, QueryBuilder, Sqlite};

pub async fn insert_media<'a, E>(media: &Media, exec: E) -> Result<InsertResult>
where
    E: Executor<'a, Database = Sqlite>,
{
    let res = sqlx::query(
        "
            INSERT INTO media (
                id, created_at
            )
            VALUES (
                ?, ?
            )
            ",
    )
    .bind(&media.id)
    .bind(media.created_at)
    .execute(exec)
    .await
    .to_app_err()?;

    Ok(res.into())
}

pub async fn insert_media_file<'a, E>(file: &MediaFile, exec: E) -> Result<InsertResult>
where
    E: Executor<'a, Database = Sqlite>,
{
    let res = sqlx::query(
        "
            INSERT INTO media_files (
                id, storage_path, media_id,
                variant, created_at, size_bytes, mimetype,
                duration_ms
            )
            VALUES (
                ?, ?, ?,
                ?, ?, ?, ?,
                ?
            )
            ",
    )
    .bind(&file.id)
    .bind(&file.storage_path)
    .bind(&file.media_id)
    .bind(file.variant)
    .bind(file.created_at)
    .bind(file.size_bytes)
    .bind(file.mimetype)
    .bind(file.duration_ms)
    .execute(exec)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(e) if e.is_unique_violation() => create_error!(AlreadyExists),
        _ => create_error!(source = e),
    })?;

    Ok(res.into())
}

pub async fn get_media_files<'a, E>(ids: &[MediaId], exec: E) -> Result<Vec<MediaFile>>
where
    E: Executor<'a, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM media_files
            WHERE media_id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });

    qb.build_query_as().fetch_all(exec).await.to_app_err()
}

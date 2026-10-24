use db_api::{DatabaseError, result::ToDatabaseError};
use models::{id::FileGroupId, types::FileKey};
use sqlx::{QueryBuilder, SqliteConnection};

use crate::schema::{DbFile, DbFileGroup};

pub async fn insert_file(file: &DbFile, conn: &mut SqliteConnection) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO files (
            group_id, variant,
            created_at,
            path,
            size_bytes, mime_type,
            duration_ms
        )
        VALUES (
            ?, ?,
            ?,
            ?,
            ?, ?,
            ?
        )
        ",
    )
    .bind(&file.group_id)
    .bind(file.variant)
    .bind(file.created_at)
    .bind(&file.path)
    .bind(file.size_bytes)
    .bind(file.mime_type)
    .bind(file.duration_ms)
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn insert_file_group(
    group: &DbFileGroup,
    conn: &mut SqliteConnection,
) -> Result<(), DatabaseError> {
    sqlx::query(
        "
        INSERT INTO file_groups (
            id
        )
        VALUES (
            ?
        )
        ",
    )
    .bind(&group.id)
    .execute(conn)
    .await
    .to_db_error()?;

    Ok(())
}

pub async fn load_file(
    key: &FileKey,
    conn: &mut SqliteConnection,
) -> Result<Option<DbFile>, DatabaseError> {
    sqlx::query_as(
        "
        SELECT * FROM files
        WHERE group_id = ? AND variant = ?
        ",
    )
    .bind(key.group_id())
    .bind(key.variant())
    .fetch_optional(conn)
    .await
    .to_db_error()
}

pub async fn load_files_by_group(
    id: &FileGroupId,
    conn: &mut SqliteConnection,
) -> Result<Vec<DbFile>, DatabaseError> {
    load_files_by_groups(std::slice::from_ref(id), conn).await
}

pub async fn load_files_by_groups(
    ids: &[FileGroupId],
    conn: &mut SqliteConnection,
) -> Result<Vec<DbFile>, DatabaseError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM files
            WHERE group_id IN (
            ",
    );

    let mut separated = qb.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

pub async fn load_file_group(
    id: &FileGroupId,
    conn: &mut SqliteConnection,
) -> Result<Option<DbFileGroup>, DatabaseError> {
    load_file_groups(std::slice::from_ref(id), conn)
        .await
        .map(|s| s.into_iter().next())
}

pub async fn load_file_groups(
    ids: &[FileGroupId],
    conn: &mut SqliteConnection,
) -> Result<Vec<DbFileGroup>, DatabaseError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM file_groups
            WHERE id IN (
            ",
    );

    let mut separated = qb.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    qb.build_query_as().fetch_all(conn).await.to_db_error()
}

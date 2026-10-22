use db_api::{
    DatabaseError,
    ports::{DeleteFn, FilesDatabase, UpdateFn},
    result::ToDatabaseError,
};
use models::{
    EntityError,
    entities::{File, FileGroup},
    id::FileGroupId,
    types::FileKey,
};
use sqlx::{Executor, QueryBuilder, Sqlite};

use crate::{
    driver::SqliteDatabase,
    schema::{DbFile, DbFileGroup},
};

pub struct SqliteFilesRepository {
    db: SqliteDatabase,
}

#[async_trait::async_trait]
impl FilesDatabase for SqliteFilesRepository {
    async fn insert_file(&self, file: &File) -> Result<(), DatabaseError> {
        let file = DbFile::from_domain(file);
        insert_file(&file, self.db.pool()).await
    }

    async fn get_file_group_by_id(
        &self,
        id: &FileGroupId,
    ) -> Result<Option<FileGroup>, DatabaseError> {
        let mut conn = self.db.acquire().await?;

        let id = std::slice::from_ref(id);

        let Some(group) = get_file_groups_by_ids(id, &mut *conn)
            .await?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };

        let files = get_files_by_groups_ids(id, &mut *conn).await?;

        let domain = DbFileGroup::into_domain(group, files)?;
        Ok(Some(domain))
    }

    async fn get_file_by_key(&self, key: &FileKey) -> Result<Option<File>, DatabaseError> {
        get_file(key, self.db.pool())
            .await?
            .map(DbFile::into_domain)
            .transpose()
    }

    async fn update_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<UpdateFn<File>>,
    ) -> Result<File, DatabaseError> {
        let mut tx = self.db.begin().await?;

        let mut file = get_file(key, &mut *tx)
            .await?
            .map(DbFile::into_domain)
            .transpose()?
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;

        f(&mut file)?;

        let db_model = DbFile::from_domain(&file);

        let updated = sqlx::query_as(
            "
            UPDATE files
            SET path = ?, size_bytes = ?, mime_type = ?, duration_ms = ?
            WHERE group_id = ? AND variant = ?
            RETURNING *
            ",
        )
        .bind(db_model.path)
        .bind(db_model.size_bytes)
        .bind(db_model.mime_type)
        .bind(db_model.duration_ms)
        .bind(key.group_id())
        .bind(key.variant())
        .fetch_one(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(DbFile::into_domain(updated)?)
    }

    async fn delete_file_fn_boxed(
        &self,
        key: &FileKey,
        f: Box<DeleteFn<File>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.db.begin().await?;

        let file = get_file(key, &mut *tx)
            .await?
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;
        let domain = DbFile::into_domain(file)?;

        f(&domain)?;

        sqlx::query(
            "
            DELETE FROM files
            WHERE group_id = ? AND variant = ?
            ",
        )
        .bind(key.group_id())
        .bind(key.variant())
        .execute(&mut *tx)
        .await
        .to_db_error()?;

        tx.commit().await.to_db_error()?;

        Ok(())
    }

    async fn delete_file_group_fn_boxed(
        &self,
        id: &FileGroupId,
        f: Box<DeleteFn<FileGroup>>,
    ) -> Result<(), DatabaseError> {
        let mut tx = self.db.begin().await?;

        let id_arr = std::slice::from_ref(id);

        let group = get_file_groups_by_ids(id_arr, &mut *tx)
            .await?
            .into_iter()
            .next()
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;
        let files = get_files_by_groups_ids(id_arr, &mut *tx).await?;

        let domain = DbFileGroup::into_domain(group, files)?;

        f(&domain)?;

        sqlx::query(
            "
            DELETE FROM file_groups
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

pub(crate) async fn insert_file<'a, E>(file: &DbFile, exec: E) -> Result<(), DatabaseError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query(
        "
        INSERT INTO files (
            group_id, variant
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
    .execute(exec)
    .await
    .to_db_error()?;

    Ok(())
}

pub(crate) async fn get_file<'a, E>(key: &FileKey, exec: E) -> Result<Option<DbFile>, DatabaseError>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query_as(
        "
        SELECT * FROM files
        WHERE group_id = ? AND variant = ?
        ",
    )
    .bind(key.group_id())
    .bind(key.variant())
    .fetch_optional(exec)
    .await
    .to_db_error()
}

pub(crate) async fn get_file_groups_by_ids<'a, E>(
    ids: &[FileGroupId],
    exec: E,
) -> Result<Vec<DbFileGroup>, DatabaseError>
where
    E: Executor<'a, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM file_groups
            WHERE id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });

    qb.build_query_as().fetch_all(exec).await.to_db_error()
}

pub(crate) async fn get_files_by_groups_ids<'a, E>(
    ids: &[FileGroupId],
    exec: E,
) -> Result<Vec<DbFile>, DatabaseError>
where
    E: Executor<'a, Database = Sqlite>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::new(
        "
            SELECT * FROM files
            WHERE group_id IN
            ",
    );
    qb.push_tuples(ids, |mut qb, id| {
        qb.push_bind(id);
    });

    qb.build_query_as().fetch_all(exec).await.to_db_error()
}

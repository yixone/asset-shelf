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

use crate::{coalesced, driver::SqliteDatabase, operations, schema::DbFile};

/// SQLite implementation of [`FilesDatabase`]
///
/// Provides database access to files and file groups, including
/// retrieving domain entities and performing transactional updates and
/// deletions
pub struct SqliteFilesRepository {
    db: SqliteDatabase,
}

#[async_trait::async_trait]
impl FilesDatabase for SqliteFilesRepository {
    async fn insert_file(&self, file: &File) -> Result<(), DatabaseError> {
        let file = DbFile::from_domain(file);

        let mut tx = self.db.begin().await?;

        operations::insert_file(&file, &mut tx).await?;

        tx.commit().await.to_db_error()?;

        Ok(())
    }

    async fn get_file_group_by_id(
        &self,
        id: &FileGroupId,
    ) -> Result<Option<FileGroup>, DatabaseError> {
        let mut conn = self.db.acquire().await?;

        let Some(group) = operations::load_file_group(id, &mut conn).await? else {
            return Ok(None);
        };
        let domain = coalesced::hydrate_file_group(group, &mut conn).await?;
        Ok(Some(domain))
    }

    async fn get_file_by_key(&self, key: &FileKey) -> Result<Option<File>, DatabaseError> {
        let mut conn = self.db.acquire().await?;
        operations::load_file(key, &mut conn)
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

        let file_row = operations::load_file(key, &mut tx)
            .await?
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;
        let mut file = DbFile::into_domain(file_row)?;

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

        let file_row = operations::load_file(key, &mut tx)
            .await?
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;
        let file = DbFile::into_domain(file_row)?;

        f(&file)?;

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

        let group = operations::load_file_group(id, &mut tx)
            .await?
            .ok_or(DatabaseError::Entity(EntityError::NotFound))?;
        let domain = coalesced::hydrate_file_group(group, &mut tx).await?;
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

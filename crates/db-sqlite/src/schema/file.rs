use std::time::Duration;

use chrono::{DateTime, Utc};
use db_api::DatabaseError;
use mime::MimeType;
use models::{
    entities::{File, FileGroup, file::FileData},
    id::FileGroupId,
    types::FileVariant,
};
use sqlx::prelude::FromRow;
use storage_types::StoragePath;

/// Persisted [`FileGroup`] data stored in the `file_groups` table
#[derive(FromRow)]
pub struct DbFileGroup {
    pub id: FileGroupId,
}

/// Persisted [`File`] data stored in the `files` table
#[derive(FromRow)]
pub struct DbFile {
    pub group_id: FileGroupId,
    pub variant: FileVariant,

    pub created_at: DateTime<Utc>,

    pub path: StoragePath,

    pub size_bytes: i64,
    pub mime_type: MimeType,

    pub duration_ms: Option<i64>,
}

impl DbFile {
    /// Converts a domain file into record for the SQLite schema
    pub fn from_domain(f: &File) -> DbFile {
        DbFile {
            group_id: f.group_id().clone(),
            variant: f.variant(),
            created_at: f.created_at(),
            path: f.path().clone(),
            size_bytes: f.size_bytes() as i64,
            mime_type: f.mime_type(),
            duration_ms: f.duration().map(|d| d.as_millis() as i64),
        }
    }

    /// Reconstructs a domain file from its persisted record
    pub fn into_domain(file: DbFile) -> Result<File, DatabaseError> {
        let file = File::from_persistence(
            (file.group_id, file.variant),
            file.created_at,
            FileData {
                path: file.path,
                size_bytes: file.size_bytes as u64,
                mime_type: file.mime_type,
                duration: file.duration_ms.map(|d| Duration::from_millis(d as u64)),
            },
        );
        Ok(file)
    }
}

impl DbFileGroup {
    /// Converts a domain file group into records for the SQLite schema
    pub fn from_domain(g: FileGroup) -> (DbFileGroup, Vec<DbFile>) {
        let group = DbFileGroup { id: g.id().clone() };
        let files = g.files().values().map(DbFile::from_domain).collect();
        (group, files)
    }

    /// Reconstructs a domain file group from its persisted records
    pub fn into_domain(group: DbFileGroup, files: Vec<DbFile>) -> Result<FileGroup, DatabaseError> {
        let files = files
            .into_iter()
            .map(DbFile::into_domain)
            .collect::<Result<Vec<_>, _>>()?;

        let group = FileGroup::from_persistence(group.id, files);

        Ok(group)
    }
}

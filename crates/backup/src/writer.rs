use std::path::PathBuf;

use db::Database;
use result::{Result, error::ResultExt};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{BACKUP_DB_FILE, BACKUP_MANIFEST_FILE, BackupManifest};

/// Storage and database backup writer
pub struct BackupWriter {
    /// Backup Manifest
    pub(crate) manifest: BackupManifest,

    /// Backup dir
    pub(crate) dir: PathBuf,

    /// Backup temp dir
    pub(crate) temp: PathBuf,
}

impl BackupWriter {
    /// Performs backup
    pub async fn backup(self, db: &Database) -> Result<BackupManifest> {
        // Creates a backup directory
        tokio::fs::create_dir_all(&self.dir).await.to_app_err()?;

        // Performs a database backup and handles the error
        if let Err(e) = self.save_db_backup(db).await {
            tracing::error!(err = ?e, "Failed to backup the database; clearing backup");

            // Deletes the backup directory
            let _ = tokio::fs::remove_dir_all(&self.dir).await;

            return Err(e);
        }

        // Saves a backup manifest and handles the error
        if let Err(e) = self.save_backup_manifest().await {
            tracing::error!(err = ?e, "Failed to save backup manifest; clearing backup");

            // Deletes the backup directory
            let _ = tokio::fs::remove_dir_all(&self.dir).await;

            return Err(e);
        }

        // Logs the creation of a backup
        tracing::info!(dir = self.dir.display().to_string(), "Backup created;");

        Ok(self.manifest)
    }

    /// Creates a database backup
    async fn save_db_backup(&self, db: &Database) -> Result<()> {
        let path = self.dir.join(BACKUP_DB_FILE);

        // Performs a database backup and handles the error
        db.backup(&path).await
    }

    /// Saves a backup manifest
    async fn save_backup_manifest(&self) -> Result<()> {
        let path = self.dir.join(BACKUP_MANIFEST_FILE);

        // Creates a manifest file
        let mut manifest_file = File::create_new(&path).await.to_app_err()?;

        // Serializes the manifest to a file
        let manifest_str = self.manifest.serialize()?;
        manifest_file
            .write_all(manifest_str.as_bytes())
            .await
            .to_app_err()?;
        manifest_file.sync_all().await.to_app_err()
    }
}

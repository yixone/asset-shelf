use std::path::PathBuf;

use chrono::Utc;
use db::Database;
use flake_id::FlakeIdGenerator;
use result::Result;

mod manifest;
pub use manifest::BackupManifest;

mod writer;
pub use writer::BackupWriter;

const BACKUP_MANIFEST_FILE: &str = "manifest";
const BACKUP_DB_FILE: &str = "database";

/// A entity for managing backups
pub struct BackupManager {
    /// FlakeIdGenerator for generating backup IDs
    flake: FlakeIdGenerator,
    /// Backup directory
    dir: PathBuf,
    /// Current application version
    version: String,
}

impl BackupManager {
    /// Creates a new [`BackupManager`]
    pub fn new(dir: impl Into<PathBuf>, version: &str) -> Self {
        BackupManager {
            // A custom generator with `node_id=0` is used
            // because the IDs do not leave the scope of `BackupManager`
            flake: FlakeIdGenerator::new(0),
            dir: dir.into(),
            version: version.to_string(),
        }
    }

    /// Performs backup
    pub async fn backup(&self, db: &Database) -> Result<BackupManifest> {
        let writer = self.writer();
        writer.backup(db).await
    }

    /// Creates a new backup writer
    fn writer(&self) -> BackupWriter {
        // Gets the backup creation time
        let timestamp = Utc::now().timestamp_millis() as u64;

        // Generates a backup ID
        let id = self.flake.get_id();

        // Gets the backup directory
        let dir = self
            .dir
            .join(format!("asset-shelf-backup-{}-{}", self.version, timestamp));

        let temp = self.dir.join(format!("temp-backup-{}", timestamp));

        BackupWriter {
            manifest: BackupManifest {
                version: self.version.clone(),
                created_at: timestamp,
            },
            dir,
            temp,
        }
    }
}

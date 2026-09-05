use flake_id::FlakeId;
use result::{Result, error::ResultExt};
use serde::{Deserialize, Serialize};

/// Backup Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// The version on which the backup was created
    pub version: String,
    /// Timestamp (in millis) of the backup creation
    pub created_at: u64,
}

impl BackupManifest {
    /// Serializes the manifest into a `toml` string
    pub fn serialize(&self) -> Result<String> {
        toml::to_string(self).to_app_err()
    }

    /// Deserializes the manifest from a `toml` string
    pub fn deserialize(s: &[u8]) -> Result<Self> {
        toml::from_slice(s).to_app_err()
    }
}

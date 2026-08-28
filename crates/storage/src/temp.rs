use std::{path::PathBuf, sync::Arc};

use flake_id::{FlakeIdGenerator, str::FlakeIdStr};

use crate::{StoragePath, backend::fs::NativeFsStorageBackend};

/// The part of the storage system responsible for handling temporary files
#[derive(Debug)]
pub(crate) struct TempSection {
    /// Backend for the temporary file storage section
    pub backend: NativeFsStorageBackend,
    /// Identifier generator for temporary files
    pub temp_id_generator: Arc<FlakeIdGenerator>,
}

impl TempSection {
    /// Generates a new temporary path in the temporary storage section
    pub fn generate_temp_path(&self) -> StoragePath {
        StoragePath::new(self.temp_id_generator.get_id_as::<FlakeIdStr>().to_string())
    }

    /// Returns the real path to the file on the local file system
    pub fn resolve_path(&self, path: &StoragePath) -> PathBuf {
        self.backend.resolve_path(path)
    }
}

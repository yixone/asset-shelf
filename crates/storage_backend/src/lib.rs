//! An abstract content storage backend that
//! hides the low-level file storage implementation

use std::{ops::Deref, path::Path, sync::Arc};

use crate::{core::result::Result, fs::FsStorageBackend};

pub mod core;

pub mod fs;
pub mod s3;

pub enum StorageBackend {
    Fs(Arc<FsStorageBackend>),
}

impl StorageBackend {
    pub async fn open_fs<P>(root: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let backend = FsStorageBackend::new(root).await?;
        Ok(StorageBackend::Fs(Arc::new(backend)))
    }
}

impl Deref for StorageBackend {
    type Target = dyn core::ops::AbstractStorageBackend + Send + Sync;

    fn deref(&self) -> &Self::Target {
        match self {
            StorageBackend::Fs(s) => s.as_ref(),
        }
    }
}

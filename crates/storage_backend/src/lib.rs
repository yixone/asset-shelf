//! An abstract content storage backend that
//! hides the low-level file storage implementation

use std::{ops::Deref, sync::Arc};

use crate::fs::FsStorageBackend;

pub mod core;

pub mod fs;
pub mod s3;

pub enum StorageBackend {
    Fs(Arc<FsStorageBackend>),
}

impl Deref for StorageBackend {
    type Target = dyn core::ops::StorageBackend + Send + Sync;

    fn deref(&self) -> &Self::Target {
        match self {
            StorageBackend::Fs(s) => s.as_ref(),
        }
    }
}

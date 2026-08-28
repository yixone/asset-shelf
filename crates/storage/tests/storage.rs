use std::{path::Path, sync::Arc};

use flake_id::FlakeIdGenerator;
use result::{Result, create_error};
use storage::{Storage, StoragePath, backend::fs::NativeFsStorageBackend, global::GlobalPathData};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

const DATA: &[u8] = &[0xFF, 0xD8, 0xFF, 0xDB, 0x67, 0x42, 0x52, 0x0, 0x0, 0x1, 0x1];

async fn open_storage(path: &Path) -> Storage {
    let host = NativeFsStorageBackend::new(path).await.unwrap();
    let flake = Arc::new(FlakeIdGenerator::new(0));
    Storage::new(host, flake, path.join("temp")).await.unwrap()
}

/// Tests saving a file to storage
#[tokio::test]
async fn storage_upload() {
    let temp = TempDir::new().unwrap();
    let storage = open_storage(temp.path()).await;

    let file = storage
        .upload(GlobalPathData::new("FTF0ulblKa", "test"), DATA, |_| Ok(()))
        .await
        .unwrap();

    let file = file.commit().await.unwrap();
    assert_eq!(file.size_bytes, DATA.len());

    {
        let mut f = storage.open(&file.global_path).await.unwrap();

        let mut buf = Vec::new();
        f.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, DATA);
    }
}

/// Tests automatic cleaning of the temporary file when a write error occurs
#[tokio::test]
async fn storage_upload_with_err() {
    let temp = TempDir::new().unwrap();
    let storage = open_storage(temp.path()).await;

    let res = storage
        .upload(GlobalPathData::new("FTF0ulblKa", "test"), DATA, |_| {
            // Simulates the occurrence of an error while writing
            Err(create_error!(UnsupportedFileType))
        })
        .await;
    assert!(res.is_err());

    let real_path = temp.path().join(
        StoragePath::new_container("FTF0ulblKa")
            .join("test")
            .to_path(),
    );

    assert!(!real_path.exists());
}

/// Tests automatic cleaning of the temporary file when a write error occurs before commit
#[tokio::test]
async fn storage_upload_with_err_before_commit() {
    let temp = TempDir::new().unwrap();
    let storage = open_storage(temp.path()).await;

    let _: Result<()> = {
        let _file = storage
            .upload(GlobalPathData::new("FTF0ulblKa", "test"), DATA, |_| Ok(()))
            .await
            .unwrap();
        Err(create_error!(UnsupportedFileType))
    };

    let real_path = temp.path().join(
        StoragePath::new_container("FTF0ulblKa")
            .join("test")
            .to_path(),
    );

    assert!(!real_path.exists());
}

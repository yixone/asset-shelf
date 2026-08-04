// TODO!
// - MV
// - MV w/ conflict

use bytes::Bytes;
use storage::backend::{StorageBackend, fs::NativeFsStorageBackend, path::StoragePath};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

const DATA: &[u8] = &[0xFF, 0xD8, 0xFF, 0xDB, 0x67, 0x42, 0x52, 0x0, 0x0, 0x1, 0x1];

async fn open_storage() -> NativeFsStorageBackend {
    let temp = TempDir::new().unwrap();
    NativeFsStorageBackend::new(temp.path()).await.unwrap()
}

#[tokio::test]
async fn create_and_read_file() {
    let storage_backend = open_storage().await;

    let path = StoragePath::new("test_file");

    {
        let mut writer = storage_backend.create(&path).await.unwrap();
        writer.write(Bytes::from_static(DATA)).await.unwrap();
        writer.flush().await.unwrap();
    }

    {
        let mut reader = storage_backend.read(&path).await.unwrap();

        let mut test_buf = Vec::new();
        reader.read_to_end(&mut test_buf).await.unwrap();

        assert_eq!(test_buf, DATA);
    }
}

#[tokio::test]
async fn abort_writting() {
    let storage_backend = open_storage().await;

    let path = StoragePath::new("test_file");

    {
        let writer = storage_backend.create(&path).await.unwrap();
        writer.abort().await.unwrap();
    }

    {
        let exists = storage_backend.exists(&path).await.unwrap();
        assert!(!exists);
    }
}

#[tokio::test]
async fn delete_with_parents() {
    let storage_backend = open_storage().await;
    let path_0 = StoragePath::new("a/b/c/d/file");
    let path_1 = StoragePath::new("a/b/c/file");

    {
        let mut writer = storage_backend.create(&path_0).await.unwrap();
        writer.write(Bytes::from_static(DATA)).await.unwrap();
        writer.flush().await.unwrap();

        let mut writer = storage_backend.create(&path_1).await.unwrap();
        writer.write(Bytes::from_static(DATA)).await.unwrap();
        writer.flush().await.unwrap();
    }

    {
        storage_backend.remove(&path_0).await.unwrap();

        assert!(!storage_backend.exists(&path_0).await.unwrap());

        // Checking that the file's parent directory was deleted
        assert!(
            !storage_backend
                .resolve_path(&StoragePath::new("a/b/c/d"))
                .exists()
        );

        // Checking that a non-empty directory located along the deletion path was not deleted
        assert!(
            storage_backend
                .resolve_path(&StoragePath::new("a/b/c"))
                .is_dir()
        );
    }
}

#[tokio::test]
async fn do_not_delete_parent_dir_if_not_empty() {
    let storage_backend = open_storage().await;
    let path_0 = StoragePath::new("a/b/c/d/file");
    let path_1 = StoragePath::new("a/b/c/d/file2");

    {
        let mut writer = storage_backend.create(&path_0).await.unwrap();
        writer.write(Bytes::from_static(DATA)).await.unwrap();
        writer.flush().await.unwrap();

        let mut writer = storage_backend.create(&path_1).await.unwrap();
        writer.write(Bytes::from_static(DATA)).await.unwrap();
        writer.flush().await.unwrap();
    }

    {
        storage_backend.remove(&path_0).await.unwrap();

        // Checking that the file's parent directory was deleted
        assert!(
            storage_backend
                .resolve_path(&StoragePath::new("a/b/c/d"))
                .exists()
        );

        assert!(storage_backend.exists(&path_1).await.unwrap());
    }
}

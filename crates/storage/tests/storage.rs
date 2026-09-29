use std::path::PathBuf;

use storage::{
    mount::StorageMountPoint, path::StoragePath, result::StorageError, storage::Storage,
};
use tempfile::TempDir;

const DATA: &[u8] = &[0x0, 0xFF, 0xD, 0xAB, 0xF, 0x67, 0x43];

async fn prepare() -> (Storage, PathBuf) {
    let temp = TempDir::new().unwrap();

    let root = StorageMountPoint::new(temp.path().to_path_buf());
    root.ensure().await.unwrap();

    let storage = Storage::new(root);
    (storage, temp.path().to_path_buf())
}

mod write {
    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn write_and_commit() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_file_writer(&path).await.unwrap();
        writer.write_reader(DATA).await.unwrap();

        let staged = writer.finish().await.unwrap();

        assert_eq!(staged.size_bytes(), DATA.len() as u64);

        assert!(!storage.exists(&path).await.unwrap());

        staged.commit().await.unwrap();

        assert!(storage.exists(&path).await.unwrap());

        let mut reader = storage.reader(path).await.unwrap();
        let mut out = Vec::new();

        reader.read_to_end(&mut out).await.unwrap();

        assert_eq!(out, DATA);
    }

    #[tokio::test]
    async fn write_and_abort() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_file_writer(&path).await.unwrap();
        writer.write_reader(DATA).await.unwrap();

        writer.abort().await.unwrap();

        assert!(!storage.exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn abort_on_drop() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_file_writer(&path).await.unwrap();
        writer.write_reader(DATA).await.unwrap();

        // Dropping the writer aborts the staged file
        drop(writer);

        assert!(!storage.exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn return_error_on_conflict() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_file_writer(&path).await.unwrap();
        writer.write_reader(DATA).await.unwrap();
        let stage = writer.finish().await.unwrap();
        stage.commit().await.unwrap();

        let err = storage.create_file_writer(&path).await.err().unwrap();
        assert!(matches!(err, StorageError::AlreadyExists))
    }

    #[tokio::test]
    async fn return_error_on_commit_conflict() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_file_writer(&path).await.unwrap();
        let mut writer2 = storage.create_file_writer(&path).await.unwrap();
        writer.write_reader(DATA).await.unwrap();
        writer2.write_reader(DATA).await.unwrap();

        let stage = writer.finish().await.unwrap();
        stage.commit().await.unwrap();

        let stage2 = writer2.finish().await.unwrap();
        let err = stage2.commit().await.err().unwrap();
        assert!(matches!(err, StorageError::AlreadyExists));
    }
}

mod read {
    // TODO!
}

mod validation {
    use super::*;

    #[tokio::test]
    async fn return_error_for_invalid_path() {
        let (storage, ..) = prepare().await;

        let path = StoragePath::new("/../../../etc");

        let res = storage.reader(path).await.err().unwrap();

        assert!(matches!(res, StorageError::InvalidPath));
    }
}

use flake_id::FlakeIdGenerator;
use storage::{Storage, backend::fs::NativeFsStorageBackend, global::GlobalPathData};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

const DATA: &[u8] = &[0xFF, 0xD8, 0xFF, 0xDB, 0x67, 0x42, 0x52, 0x0, 0x0, 0x1, 0x1];

async fn open_storage() -> Storage {
    let temp = TempDir::new().unwrap();
    let host = NativeFsStorageBackend::new(temp.path()).await.unwrap();
    let flake = FlakeIdGenerator::new(0);
    Storage::new(host, flake, temp.path().join("temp"))
        .await
        .unwrap()
}

#[tokio::test]
async fn storage_upload() {
    let storage = open_storage().await;

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

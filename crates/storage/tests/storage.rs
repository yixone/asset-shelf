use flake_id::FlakeIdGenerator;
use storage::{Storage, backend::fs::NativeFsStorageBackend};
use tempfile::TempDir;

const DATA: &[u8] = &[0xFF, 0xD8, 0xFF, 0xDB, 0x67, 0x42, 0x52, 0x0, 0x0, 0x1, 0x1];
const NAMEPSACE: &str = "test";

async fn open_storage() -> Storage {
    let temp = TempDir::new().unwrap();
    let host = NativeFsStorageBackend::new(temp.path()).await.unwrap();
    let flake = FlakeIdGenerator::new(0);
    Storage::new(host, flake, 1024 * 1024)
}

#[tokio::test]
async fn storage_upload() {
    let storage = open_storage().await;

    let file = storage
        .upload(NAMEPSACE, "FTF0ulblKa", "test", DATA)
        .await
        .unwrap();

    let file = file.release().await.unwrap();
    assert_eq!(file.size_bytes, DATA.len());
}

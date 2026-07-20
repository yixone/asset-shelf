use flake_id::FlakeIdGenerator;
use mimetype::MimeType;
use storage::Storage;
use storage_backend::StorageBackend;
use tempfile::TempDir;

async fn prepare() -> Storage {
    let temp = TempDir::new().unwrap();
    let host = StorageBackend::open_fs(temp.path()).await.unwrap();
    let flake = FlakeIdGenerator::new(1);
    Storage::new(host, flake, 1024 * 1024)
}

const TEST_BLOB: &[u8] = &[0xFF, 0xD8, 0xFF, 0xDB, 0x67, 0x42, 0x52, 0x0, 0x0, 0x1, 0x1];
const TEST_NAMEPSACE: &str = "test";

#[tokio::test]
async fn upload_file_to_storage() {
    let storage = prepare().await;

    {
        let res = storage.upload(TEST_NAMEPSACE, TEST_BLOB).await.unwrap();
        assert_eq!(res.size_bytes, TEST_BLOB.len());
        assert_eq!(res.mimetype, MimeType::Jpeg);
        assert_eq!(res.path.namespace, TEST_NAMEPSACE);
    }
}

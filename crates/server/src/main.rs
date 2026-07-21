use std::sync::Arc;

use db::sqlite::SqliteDb;
use flake_id::FlakeIdGenerator;
use result::Result;
use server::SERVER_VERSION;
use storage::Storage;
use storage_backend::StorageBackend;
use tokio_util::sync::CancellationToken;
use workers::{di::WorkerContext, supervisor::WorkersSupervisor, units::cleanup::CleanupWorker};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Asset shelf server - {} :3", SERVER_VERSION);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();

    let db = SqliteDb::open("storage/data.db").await?;
    db.migrate().await?;

    let storage_backend = StorageBackend::open_fs("storage/data").await?;

    let storage = Arc::new(Storage::new(
        storage_backend,
        FlakeIdGenerator::new(1),
        8 * 1024 * 1024 * 1024,
    ));

    let workers_context = WorkerContext { db, storage };
    let (cleanup_events, cleanup_worker) = CleanupWorker::new(workers_context.clone());

    let cancel = CancellationToken::new();

    WorkersSupervisor::new()
        .with_worker(cleanup_worker)
        .run(cancel.clone());

    Ok(())
}

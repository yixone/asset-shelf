use std::sync::Arc;

use db::sqlite::SqliteDb;
use flake_id::FlakeIdGenerator;
use result::Result;
use server::{
    SERVER_VERSION,
    di::{DataCtx, EventsContext},
};
use storage::Storage;
use storage_backend::StorageBackend;
use tokio_util::sync::CancellationToken;
use workers::{di::WorkerContext, supervisor::WorkersSupervisor, units::cleanup::CleanupWorker};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    print_header();

    let db = Arc::new(SqliteDb::open("storage/data.db").await?);
    db.migrate().await?;

    let storage_backend = StorageBackend::open_fs("storage/data").await?;
    let storage = Arc::new(Storage::new(
        storage_backend,
        FlakeIdGenerator::new(1),
        8 * 1024 * 1024 * 1024,
    ));

    let ctx = DataCtx { db, storage };

    let cancel = CancellationToken::new();
    let (supervisor, events) = init_workers(&ctx);

    supervisor.run(cancel.clone());

    Ok(())
}

fn print_header() {
    tracing::info!("Asset shelf server - {SERVER_VERSION} :3");
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();
}

fn init_workers(ctx: &DataCtx) -> (WorkersSupervisor, EventsContext) {
    let workers_context = WorkerContext {
        db: ctx.db.clone(),
        storage: ctx.storage.clone(),
    };

    let (cleanup_events, cleanup_worker) = CleanupWorker::new(workers_context.clone());

    (
        WorkersSupervisor::new().with_worker(cleanup_worker),
        EventsContext {
            cleanup: cleanup_events,
        },
    )
}

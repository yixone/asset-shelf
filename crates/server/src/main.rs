use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use db::sqlite::SqliteDb;
use flake_id::FlakeIdGenerator;
use result::{Result, error::ResultExt};
use server::{
    SERVER_VERSION,
    di::{DataCtx, EventsContext},
    routes,
};
use storage::Storage;
use storage_backend::StorageBackend;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use workers::{
    di::WorkerContext,
    supervisor::{SupervisorHandle, WorkersSupervisor},
    units::cleanup::CleanupWorker,
};

const HOST_ADDR: &str = "0.0.0.0:8080";

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

    let flake = Arc::new(FlakeIdGenerator::new(2));
    let ctx = DataCtx { db, storage, flake };

    let cancel = CancellationToken::new();
    let (supervisor, events) = init_workers(&ctx);

    let workers_handle = supervisor.run(cancel.clone());

    tracing::info!("Server started on http://{HOST_ADDR}!");
    tracing::info!("API documentation is available at http://{HOST_ADDR}/docs/");

    let server = configure_server(ctx, events)?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, workers_handle);

    server.await.to_app_err()
}

fn configure_server(ctx: DataCtx, events: EventsContext) -> Result<actix_web::dev::Server> {
    let ctx = web::Data::new(ctx);
    let events = web::Data::new(events);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(ctx.clone())
            .app_data(events.clone())
            .configure(routes::cfg)
    })
    .bind(HOST_ADDR)
    .to_app_err()?
    .run())
}

fn spawn_shutdown_handler(
    cancel: CancellationToken,
    server_handle: ServerHandle,
    workers_handle: SupervisorHandle,
) {
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => tracing::info!("Starting graceful shutdown..."),
            Err(e) => {
                tracing::error!(err = ?e, "Failed to wait for ^C");
                return;
            }
        }
        server_handle.stop(true).await;
        cancel.cancel();
        workers_handle.stop().await;
    });
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

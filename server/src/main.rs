use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use db::sqlite::SqliteDatabase;
use flake_id::FlakeIdGenerator;
use result::{Result, error::ResultExt};
use server::{SERVER_VERSION, di::DataCtx, routes};
use storage::Storage;
use storage_backend::StorageBackend;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use workers::{
    di::WorkerContext,
    events::bus::EventBus,
    supervisor::{SupervisorHandle, WorkersSupervisor},
    units::{cleanup::CleanupWorker, media::MediaWorker},
};

const HOST_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    print_header();

    tracing::info!("Opening database...");
    let db = Arc::new(SqliteDatabase::open("storage/data.db").await?);
    db.migrate().await?;

    tracing::info!("Opening storage...");
    let storage_backend = StorageBackend::open_fs("storage/data").await?;
    let storage = Arc::new(Storage::new(
        storage_backend,
        FlakeIdGenerator::new(1),
        8 * 1024 * 1024 * 1024,
    ));

    let events = Arc::new(EventBus::new(1024));
    let flake = Arc::new(FlakeIdGenerator::new(2));
    let ctx = DataCtx {
        db,
        storage,
        flake,
        events,
    };

    let cancel = CancellationToken::new();
    let supervisor = init_workers(&ctx);

    let workers_handle = supervisor.run(cancel.clone());

    tracing::info!("Server started on http://{HOST_ADDR}!");

    let server = configure_server(ctx)?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, workers_handle);

    server.await.to_app_err()?;

    tracing::info!("Server closed!");
    Ok(())
}

fn configure_server(ctx: DataCtx) -> Result<actix_web::dev::Server> {
    let ctx = web::Data::new(ctx);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(ctx.clone())
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
            Ok(_) => {
                println!();
                tracing::info!("Starting graceful shutdown...")
            }
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
    tracing::info!("Asset shelf server - {SERVER_VERSION}");
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();
}

fn init_workers(ctx: &DataCtx) -> WorkersSupervisor {
    let workers_context = WorkerContext {
        db: ctx.db.clone(),
        storage: ctx.storage.clone(),
        flake: ctx.flake.clone(),
        events: ctx.events.clone(),
    };

    let cleanup_worker = CleanupWorker::new(workers_context.clone());
    let media_worker = MediaWorker::new(workers_context.clone());

    WorkersSupervisor::new()
        .with_worker(cleanup_worker)
        .with_worker(media_worker)
}

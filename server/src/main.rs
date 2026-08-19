use std::{path::PathBuf, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use db::sqlite::driver::SqliteDatabase;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use result::{Result, error::ResultExt};
use server::{SERVER_VERSION, di::DataCtx, routes};
use storage::{Storage, backend::fs::NativeFsStorageBackend};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use workers::{
    cleanup::CleanupWorker,
    media::MediaWorker,
    runtime::{SupervisorHandle, WorkerContext, WorkersSupervisor},
};

const HOST_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    print_header();

    tracing::info!("Opening database");
    let db = SqliteDatabase::open("storage/data.db").await?;
    db.migrate().await?;

    let db = Arc::new(db.repositories());

    tracing::info!("Opening storage");
    let storage_backend = NativeFsStorageBackend::new("storage/global").await?;
    let storage = Arc::new(
        Storage::new(
            storage_backend,
            FlakeIdGenerator::new(1),
            PathBuf::from("storage").join("temp"),
        )
        .await?,
    );

    tracing::info!("Initializing event bus");
    let events = Arc::new(EventBus::new(1024));
    let cancel = CancellationToken::new();

    let flake = Arc::new(FlakeIdGenerator::new(2));
    let ctx = DataCtx {
        db,
        storage,
        flake,
        events,
    };

    tracing::info!("Initializing background worker supervisor");
    let supervisor = init_workers(&ctx);
    let workers_handle = supervisor.run(cancel.clone());

    let server = configure_server(ctx)?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, workers_handle);

    tracing::info!("Server started on http://{HOST_ADDR}!");
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
    tracing::info!("{}", "=".repeat(18));
    tracing::info!("Asset shelf server");
    tracing::info!("Version: {SERVER_VERSION}");
    tracing::info!("{}", "=".repeat(18));
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .with_target(false)
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

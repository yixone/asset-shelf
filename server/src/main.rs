use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use db::sqlite::SqliteDatabase;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use result::{Result, error::ResultExt};
use server::{
    SERVER_VERSION,
    di::{DataCtx, MetricsCtx},
    load_config, load_library, routes,
};
use storage::{Storage, backend::fs::NativeFsStorageBackend};
use telemetry::MetricsRegistry;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use workers::{
    cleanup::CleanupWorker,
    media::MediaWorker,
    runtime::{SupervisorHandle, WorkerContext, WorkersSupervisor},
};

#[tokio::main]
async fn main() -> Result<()> {
    let (lib_path, lib) = load_library()?;

    init_tracing();
    print_header();

    tracing::info!(path = ?lib_path, "Library loaded");

    let cfg = load_config(lib_path.join(lib.config_path()))?;

    let metrics_reg = MetricsRegistry::new(cfg.instance.allow_metrics());
    let metrics_ctx = MetricsCtx::try_new(metrics_reg)?;

    tracing::info!("Opening database");
    let db = SqliteDatabase::open(cfg.database.path()).await?;
    db.migrate().await?;

    let db = Arc::new(db.repositories());

    let flake = Arc::new(FlakeIdGenerator::new(lib.lib_id()));

    tracing::info!("Opening storage");
    let storage_backend = NativeFsStorageBackend::new(cfg.storage.dir()).await?;
    let storage = Arc::new(Storage::new(storage_backend, flake.clone(), cfg.storage.temp()).await?);

    tracing::info!("Initializing event bus");
    let events = Arc::new(EventBus::new(1024));
    let cancel = CancellationToken::new();

    let ctx = DataCtx {
        db,
        storage,
        flake,
        events,
        config: cfg.clone(),
    };

    tracing::info!("Initializing background worker supervisor");
    let supervisor = init_workers(&ctx);
    let workers_handle = supervisor.run(cancel.clone());

    let server = configure_server(ctx, metrics_ctx, &cfg.server.listen_addr())?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, workers_handle);

    tracing::info!("Server started on http://{}!", cfg.server.listen_addr());
    server.await.to_app_err()?;

    tracing::info!("Server closed!");
    Ok(())
}

fn configure_server(
    ctx: DataCtx,
    metrics: MetricsCtx,
    host_addr: &str,
) -> Result<actix_web::dev::Server> {
    let ctx = web::Data::new(ctx);
    let metrics = web::Data::new(metrics);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(ctx.clone())
            .app_data(metrics.clone())
            .configure(routes::cfg)
    })
    .bind(host_addr)
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
    tracing::info!("{}", "=".repeat(26));
    tracing::info!("Asset shelf server ({SERVER_VERSION})");
    tracing::info!("{}", "=".repeat(26));
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

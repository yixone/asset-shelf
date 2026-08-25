use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use config::{DatabaseDriverConfig, StorageBackendConfig};
use db::sqlite::SqliteDatabase;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use result::{Result, error::ResultExt};
use server::{
    SERVER_VERSION,
    di::{DataCtx, MetricsCtx},
    load_config, routes,
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
    init_tracing();
    print_header();

    let cfg = load_config()?;

    let metrics_reg = MetricsRegistry::new(cfg.instance.telemetry.enabled());
    let metrics_ctx = MetricsCtx::try_new(metrics_reg)?;
    if cfg.instance.telemetry.enabled() {
        tracing::info!("Telemetry is enabled");
    }

    let flake = Arc::new(FlakeIdGenerator::new(cfg.instance.node_id()));

    tracing::info!("Initializing dependencies");
    let db = match cfg.database.driver() {
        DatabaseDriverConfig::Sqlite { path } => {
            tracing::info!(path = path, "|- Using SQLITE database:");

            let db = SqliteDatabase::open(path).await?;
            db.migrate().await?;

            Arc::new(db.repositories())
        }
    };

    let storage = match cfg.storage.backend() {
        StorageBackendConfig::Native { dir, temp } => {
            tracing::info!(dir = dir, "|- Using NATIVE storage:");

            let storage_backend = NativeFsStorageBackend::new(dir).await?;
            Arc::new(Storage::new(storage_backend, flake.clone(), temp.into()).await?)
        }
    };

    tracing::info!("Initializing the background infrastructure");
    tracing::info!("|- Initializing event bus");
    let events = Arc::new(EventBus::new(1024));
    let cancel = CancellationToken::new();

    let ctx = DataCtx {
        db,
        storage,
        flake,
        events,
        config: cfg.clone(),
    };

    tracing::info!("|- Initializing background worker supervisor");
    let supervisor = init_workers(&ctx);
    let workers_handle = supervisor.run(cancel.clone());

    tracing::info!("Preparing the server");
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
                tracing::info!("Starting graceful shutdown")
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

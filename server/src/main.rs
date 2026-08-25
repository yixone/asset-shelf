use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use db::sqlite::SqliteDatabase;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use instance::{
    config::AppConfig,
    library::{LibDatabase, LibStorage},
};
use result::{Result, error::ResultExt};
use server::{
    SERVER_VERSION,
    di::{DataCtx, LibraryCtx, MetricsCtx},
    init_metrics, load_library, routes,
};
use storage::{Storage, backend::fs::NativeFsStorageBackend};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use workers::{
    cleanup::CleanupWorker,
    media::MediaWorker,
    runtime::{SupervisorHandle, WorkerContext, WorkersSupervisor},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut cfg = AppConfig::load("config.toml").to_app_err()?;
    let (lib, libs) = load_library(&cfg)?;

    if cfg.selected_lib_path().is_none() {
        cfg.set_selected_lib_path(lib.dir.display().to_string());
        cfg.write("config.toml").to_app_err()?;
    }

    let cfg = Arc::new(cfg);

    init_tracing();
    print_header();

    tracing::info!(path = ?lib.dir, name = ?lib.manifest.name(), "Library loaded:");

    let metrics_ctx = init_metrics(cfg.telemetry_enabled())?;

    let node_id = lib.manifest.lib_id();
    let flake = Arc::new(FlakeIdGenerator::new(node_id));

    let db = match lib.manifest.database() {
        LibDatabase::Sqlite { path } => {
            let path = lib.dir.join(path);

            tracing::info!(path = ?path, "Using sqlite database:");

            let db = SqliteDatabase::open(&path).await?;
            db.migrate().await?;

            Arc::new(db.repositories())
        }
    };

    let storage = match lib.manifest.storage() {
        LibStorage::Native { dir, temp } => {
            let dir = lib.dir.join(dir);
            let temp = lib.dir.join(temp);

            tracing::info!(dir = ?dir, "Using native storage:");

            let storage_backend = NativeFsStorageBackend::new(dir).await?;
            Arc::new(Storage::new(storage_backend, flake.clone(), temp).await?)
        }
    };

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

    let libs = LibraryCtx {
        active: lib,
        available: libs,
    };

    tracing::info!("Initializing background worker supervisor");
    let supervisor = init_workers(&ctx);
    let workers_handle = supervisor.run(cancel.clone());

    let addr = format!("0.0.0.0:{}", cfg.listen_port());

    let server = configure_server(ctx, metrics_ctx, libs, &addr)?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, workers_handle);

    tracing::info!("Server started on http://{}!", addr);
    server.await.to_app_err()?;

    tracing::info!("Server closed!");

    Ok(())
}

fn configure_server(
    ctx: DataCtx,
    metrics: MetricsCtx,
    libs: LibraryCtx,
    host_addr: &str,
) -> Result<actix_web::dev::Server> {
    let ctx = web::Data::new(ctx);
    let metrics = web::Data::new(metrics);
    let libs = web::Data::new(libs);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(ctx.clone())
            .app_data(metrics.clone())
            .app_data(libs.clone())
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

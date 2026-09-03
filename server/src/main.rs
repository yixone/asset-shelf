use std::{sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::ServerHandle, web};
use config::{DatabaseDriverConfig, StorageBackendConfig};
use db::sqlite::SqliteDatabase;
use events::EventBus;
use flake_id::FlakeIdGenerator;
use jobs::{Job, JobContext};
use jobs_runtime::{JobSchedule, JobsDispatcher, JobsResolver, ResolverTasksHandle};
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

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    tracing::info!(version = SERVER_VERSION, "Starting the server:");

    let cfg = load_config()?;

    let metrics_reg = MetricsRegistry::new(cfg.instance.telemetry.enabled());
    let metrics_ctx = MetricsCtx::try_new(metrics_reg)?;
    if cfg.instance.telemetry.enabled() {
        tracing::info!("Telemetry is enabled");
    }

    let flake = Arc::new(FlakeIdGenerator::new(cfg.instance.node_id()));

    let db = match cfg.database.driver() {
        DatabaseDriverConfig::Sqlite { path } => {
            let db = SqliteDatabase::open(path).await?;
            tracing::info!("Opening SQLite from file: {path}");
            db.migrate().await?;

            Arc::new(db.repositories())
        }
    };

    let storage = match cfg.storage.backend() {
        StorageBackendConfig::Native { dir, temp } => {
            let storage_backend = NativeFsStorageBackend::new(dir).await?;
            tracing::info!("Opening native storage from directory: {dir}");

            Arc::new(Storage::new(storage_backend, flake.clone(), temp.into()).await?)
        }
    };

    let cancel = CancellationToken::new();

    tracing::info!("Initializing event bus");
    let events = Arc::new(EventBus::new(1024));

    tracing::info!("Initializing background jobs");

    let resolver = JobsResolver::<Job>::builder()
        .dispatcher(JobsDispatcher::new())
        .context(JobContext {
            db: db.clone(),
            storage: storage.clone(),
            flake: flake.clone(),
        })
        .cancel(cancel.clone())
        .build_shared()
        .to_app_err()?;
    tracing::info!("Jobs resolver created");
    tracing::info!("Set workers count: {}", resolver.workers_count());

    resolver.schedule(
        Job::CleanupStorageMedia,
        JobSchedule::interval(Duration::from_mins(30)),
    );
    resolver.schedule(
        Job::ProcessUnprocessedAssets,
        JobSchedule::interval(Duration::from_mins(50)),
    );
    tracing::info!("Jobs have been added to the scheduler");

    let jobs_resolver_handle = resolver.run();
    tracing::info!("Jobs resolver runned!");

    let ctx = DataCtx {
        db,
        storage,
        flake: flake.clone(),
        jobs: resolver,
        events,
        config: cfg.clone(),
    };

    let server = configure_server(ctx, metrics_ctx, &cfg.server.listen_addr())?;
    let handle = server.handle();

    spawn_shutdown_handler(cancel, handle, jobs_resolver_handle);

    tracing::info!("Server configurated");
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
    jobs_resolver_handle: ResolverTasksHandle<Job>,
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
        jobs_resolver_handle.close().await;
    });
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .with_target(false)
        .init();
}

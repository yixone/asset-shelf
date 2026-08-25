use instance::{config::AppConfig, library::Library};
use result::{Result, error::ResultExt};
use telemetry::MetricsRegistry;

use crate::di::MetricsCtx;

pub mod dto;
pub mod middlewares;
pub mod routes;
pub mod utils;

pub mod di;
pub mod metrics;

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn load_library(cfg: &AppConfig) -> Result<(Library, Vec<Library>)> {
    let libs = Library::load_from_dir("./").to_app_err()?;

    if let Some(path) = cfg.selected_lib_path() {
        let lib = Library::load(path).to_app_err()?;
        return Ok((lib, libs));
    }

    if let Some(first) = libs.first().cloned() {
        return Ok((first, libs));
    }

    let lib = Library::default();
    tracing::info!("Library not found! Creating a new one!");
    lib.save().to_app_err()?;

    Ok((lib.clone(), vec![lib]))
}

pub fn init_metrics(allow_metrics: bool) -> Result<MetricsCtx> {
    let metrics_reg = MetricsRegistry::new(allow_metrics);
    MetricsCtx::try_new(metrics_reg)
}

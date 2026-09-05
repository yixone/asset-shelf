use std::sync::Arc;

use config::ApplicationConfig;
use result::{Result, error::ResultExt};

pub mod dto;
pub mod middlewares;
pub mod routes;
pub mod utils;

pub mod di;
pub mod metrics;

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONFIG_PATH: &str = "storage/config.toml";

/// Loads the application configuration from the file
pub fn load_config() -> Result<Arc<ApplicationConfig>> {
    tracing::info!("Reading config from `{CONFIG_PATH}`");
    let cfg = ApplicationConfig::try_load(CONFIG_PATH, true).to_app_err()?;
    Ok(Arc::new(cfg))
}

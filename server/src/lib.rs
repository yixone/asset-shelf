pub mod dto;
pub mod middleware;
pub mod routes;
pub mod utils;

pub mod di;
pub mod metrics;

pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

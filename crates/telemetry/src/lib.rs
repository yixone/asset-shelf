#![allow(clippy::new_without_default)]

mod registry;
pub use registry::MetricsRegistry;

mod adapter;
pub use adapter::ApiTelemetryAdapter;

pub(crate) mod helpers;

pub(crate) mod models;
pub use models::MetricApi;

pub(crate) mod result;

pub(crate) mod instruments;
pub use instruments::*;

// prometheus lib re-exports
pub use prometheus::{HistogramOpts, Opts};

#![allow(clippy::new_without_default)]

mod registry;
pub use registry::MetricsRegistry;

mod adapter;
pub use adapter::ApiTelemetryAdapter;

pub(crate) mod models;
pub use models::MetricApi;

pub(crate) mod result;

// prometheus lib re-exports
pub use prometheus::{Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec};
pub use prometheus::{HistogramOpts, Opts};

//! A set of wrappers for [prometheus] metrics tools for Noop implementation

use prometheus::{HistogramOpts, Opts};

use crate::MetricsRegistry;

/// An instrument that records increasing values
#[derive(Clone)]
pub enum CounterVec {
    /// [`prometheus`] counter
    Prometheus(prometheus::CounterVec),
    /// Noop counter
    Noop,
}

impl CounterVec {
    /// Creates a new [`CounterVec`]
    pub fn create(opts: Opts, labels: &[&str], reg: &MetricsRegistry) -> prometheus::Result<Self> {
        // Creates a noop tool if metric collection is disabled at the provider level
        if !reg.metrics_enabled() {
            return Ok(Self::Noop);
        }

        // Returns the Prometheus counter
        let counter = prometheus::CounterVec::new(opts, labels)?;
        reg.register(&counter)?;

        Ok(Self::Prometheus(counter))
    }

    /// Increments the value of the current counter
    pub fn inc(&self, labels: &[&str]) {
        match self {
            CounterVec::Prometheus(metric_vec) => {
                metric_vec.with_label_values(labels).inc();
            }
            CounterVec::Noop => (),
        }
    }
}

/// An instrument that records a distribution of values
#[derive(Clone)]
pub enum HistogramVec {
    /// [`prometheus`] histogram
    Prometheus(prometheus::HistogramVec),
    /// Noop histogram
    Noop,
}

impl HistogramVec {
    /// Creates a new [`CounterVec`]
    pub fn create(
        opts: HistogramOpts,
        label_names: &[&str],
        reg: &MetricsRegistry,
    ) -> prometheus::Result<Self> {
        // Creates a noop tool if metric collection is disabled at the provider level
        if !reg.metrics_enabled() {
            return Ok(Self::Noop);
        }

        // Returns the Prometheus histogram
        let histogram = prometheus::HistogramVec::new(opts, label_names)?;
        reg.register(&histogram)?;

        Ok(HistogramVec::Prometheus(histogram))
    }

    /// Add a single observation to the [`HistogramVec`]
    pub fn observe(&self, value: f64, labels: &[&str]) {
        match self {
            HistogramVec::Prometheus(metric_vec) => {
                metric_vec.with_label_values(labels).observe(value);
            }
            HistogramVec::Noop => (),
        }
    }
}

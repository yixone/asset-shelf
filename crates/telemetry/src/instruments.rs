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
        if !reg.is_metrics_enabled() {
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
            CounterVec::Prometheus(metric) => {
                metric.with_label_values(labels).inc();
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
        if !reg.is_metrics_enabled() {
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
            HistogramVec::Prometheus(metric) => {
                metric.with_label_values(labels).observe(value);
            }
            HistogramVec::Noop => (),
        }
    }
}

/// An instrument that records independent values
#[derive(Clone)]
pub enum GaugeVec {
    /// [`prometheus`] gauge
    Prometheus(prometheus::GaugeVec),
    /// Noop gauge
    Noop,
}

impl GaugeVec {
    /// Creates a new [`GaugeVec`]
    pub fn create(
        opts: Opts,
        label_names: &[&str],
        reg: &MetricsRegistry,
    ) -> prometheus::Result<Self> {
        // Creates a noop tool if metric collection is disabled at the provider level
        if !reg.is_metrics_enabled() {
            return Ok(Self::Noop);
        }

        // Returns the Prometheus gauge
        let gauge = prometheus::GaugeVec::new(opts, label_names)?;
        reg.register(&gauge)?;

        Ok(GaugeVec::Prometheus(gauge))
    }

    /// Increase the gauge by 1
    pub fn inc(&self, labels: &[&str]) {
        match self {
            GaugeVec::Prometheus(metric) => {
                metric.with_label_values(labels).inc();
            }
            GaugeVec::Noop => (),
        }
    }

    /// Decrease the gauge by 1
    pub fn dec(&self, labels: &[&str]) {
        match self {
            GaugeVec::Prometheus(metric) => {
                metric.with_label_values(labels).dec();
            }
            GaugeVec::Noop => (),
        }
    }

    /// Add the given value to the gauge
    /// (The value can be negative, resulting in a decrement of the gauge)
    pub fn add(&self, value: f64, labels: &[&str]) {
        match self {
            GaugeVec::Prometheus(metric) => {
                metric.with_label_values(labels).add(value);
            }
            GaugeVec::Noop => (),
        }
    }

    /// Subtract the given value from the gauge
    /// (The value can be negative, resulting in an increment of the gauge)
    pub fn sub(&self, value: f64, labels: &[&str]) {
        match self {
            GaugeVec::Prometheus(metric) => {
                metric.with_label_values(labels).sub(value);
            }
            GaugeVec::Noop => (),
        }
    }
}

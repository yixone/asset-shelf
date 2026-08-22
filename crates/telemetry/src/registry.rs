use prometheus::{HistogramOpts, Opts, Registry, core::Collector, proto::MetricFamily};
use result::{Result, error::ResultExt};

use crate::{CounterVec, GaugeVec, HistogramVec};

/// Application metrics registry
pub struct MetricsRegistry {
    metrics_enabled: bool,
    inner: Registry,
}

impl MetricsRegistry {
    /// Creates a new [`MetricsRegistry`]
    pub fn new(metrics_enabled: bool) -> Self {
        if metrics_enabled {
            tracing::info!("Metrics are enabled for the current instance")
        }

        Self {
            metrics_enabled,
            inner: Registry::new(),
        }
    }

    /// Calls the Collect method of the registered Collectors and then gathers the collected metrics
    pub fn gather(&self) -> Vec<MetricFamily> {
        self.inner.gather()
    }

    /// Registers a new [`Collector`] to be included in metrics collection
    pub fn register<C>(&self, collector: &C) -> prometheus::Result<()>
    where
        C: Collector + Clone + 'static,
    {
        let boxed = Box::new(collector.clone());
        self.inner.register(boxed)
    }

    /// Registers a [`HistogramVec`] metric for the current registry.
    pub fn reg_histogram_vec(
        &self,
        opts: HistogramOpts,
        label_names: &[&str],
    ) -> Result<HistogramVec> {
        HistogramVec::create(opts, label_names, self).to_app_err()
    }

    /// Registers a [`CounterVec`] metric for the current registry
    pub fn reg_counter_vec(&self, opts: Opts, label_names: &[&str]) -> Result<CounterVec> {
        CounterVec::create(opts, label_names, self).to_app_err()
    }

    /// Registers a [`GaugeVec`] metric for the current registry
    pub fn reg_gauge_vec(&self, opts: Opts, label_names: &[&str]) -> Result<GaugeVec> {
        GaugeVec::create(opts, label_names, self).to_app_err()
    }

    /// Returns `true` if metrics are enabled for this registry.
    /// Otherwise, returns `false`
    pub fn is_metrics_enabled(&self) -> bool {
        self.metrics_enabled
    }
}

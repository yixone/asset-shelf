use prometheus::{
    CounterVec, HistogramOpts, HistogramVec, Opts, Registry, core::Collector, proto::MetricFamily,
};

/// Application metrics registry
pub struct MetricsRegistry {
    metrics_enabled: bool,

    inner: Registry,
}

impl MetricsRegistry {
    /// Creates a new [`MetricsRegistry`]
    pub fn new(metrics_enabled: bool) -> Self {
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
    pub fn register<C>(&self, collector: C) -> prometheus::Result<C>
    where
        C: Collector + Clone + 'static,
    {
        let boxed = Box::new(collector.clone());
        self.inner.register(boxed)?;
        Ok(collector)
    }

    /// Registers a [`HistogramVec`] metric for the current registry.
    pub fn reg_histogram_vec(
        &self,
        opts: HistogramOpts,
        label_names: &[&str],
    ) -> prometheus::Result<HistogramVec> {
        let histogram = HistogramVec::new(opts, label_names)?;
        self.register(histogram)
    }

    /// Registers a [`CounterVec`] metric for the current registry
    pub fn reg_counter_vec(
        &self,
        opts: Opts,
        label_names: &[&str],
    ) -> prometheus::Result<CounterVec> {
        let counter = CounterVec::new(opts, label_names)?;
        self.register(counter)
    }

    pub fn metrics_enabled(&self) -> bool {
        self.metrics_enabled
    }
}

use prometheus::{Registry, core::Collector, proto::MetricFamily};

pub struct MetricsRegistry {
    inner: Registry,
}

impl MetricsRegistry {
    /// Creates a new [`MetricsRegistry`]
    pub fn new() -> Self {
        Self {
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
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

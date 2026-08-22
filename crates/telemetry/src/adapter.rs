use prometheus::proto::{Bucket, MetricFamily, MetricType};

use crate::{
    MetricApi,
    models::{HistogramBucket, HistogramBucketBound, MetricData, SummaryQuantile},
    result::AdapterError,
};

/// Adapter for formatting [`prometheus`] telemetry
/// and serving it via the API
///
/// ### Usage
/// ``` no_run
/// use telemetry::{ApiTelemetryAdapter, MetricsRegistry};
///
/// fn to_api_metrics(reg: &MetricsRegistry) -> Vec<MetricApi> {
///     // Retrieves metrics from the registry
///     let collected = reg.gather();
///
///     // Creates a new adapter
///     let adapter = ApiTelemetryAdapter::new()
///     
///     // Adapts collected metrics for MetricApi
///     adapter.to_api(&collected).unwrap()
/// }
/// ```
pub struct ApiTelemetryAdapter;

impl ApiTelemetryAdapter {
    /// Creates a new [`ApiTelemetryAdapter`]
    pub fn new() -> Self {
        ApiTelemetryAdapter
    }

    /// Accepts a slice of [`MetricFamily`]
    /// and adapts it for exposure via the API
    pub fn to_api(&self, mfs: &[MetricFamily]) -> Result<Vec<MetricApi>, AdapterError> {
        // Creates a list of metrics
        let mut list = Vec::with_capacity(mfs.len());

        for mf in mfs {
            // Fail-fast checks
            validate_metric_family(mf)?;

            // Retrieves the basic metadata of the metric
            let name = mf.name();
            let help = mf.help();
            let metric_type = mf.get_field_type();

            // Creates a new metric model
            let mut item = MetricApi::new(name, help, metric_type);

            // Add metric fields to the created item
            for m in mf.get_metric() {
                // Creates a new metric data builder
                let metric = MetricData::builder().with_labels(m.get_label());

                match metric_type {
                    MetricType::COUNTER => {
                        item.push_metric(metric.counter(m.get_counter().value()));
                    }
                    MetricType::GAUGE => {
                        item.push_metric(metric.gauge(m.get_gauge().value()));
                    }
                    MetricType::SUMMARY => {
                        let s = m.get_summary();

                        let mut quantiles = Vec::new();
                        for qt in s.get_quantile().iter() {
                            quantiles.push(SummaryQuantile {
                                quantile: qt.quantile(),
                                value: qt.value(),
                            });
                        }

                        let sum = s.sample_sum();
                        let count = s.sample_count();

                        item.push_metric(metric.summary(quantiles, sum, count));
                    }
                    MetricType::UNTYPED => {
                        item.push_metric(metric.untyped(m.untyped.value()));
                    }
                    MetricType::HISTOGRAM => {
                        let h = m.get_histogram();

                        let buckets = adapt_hist_buckets(h.get_bucket(), h.get_sample_count());

                        let sum = h.get_sample_sum();
                        let count = h.get_sample_count();

                        item.push_metric(metric.histogram_pretty(buckets, sum, count));
                    }
                }
            }

            list.push(item);
        }

        Ok(list)
    }
}

/// Validates the received [`MetricFamily`]
fn validate_metric_family(mf: &MetricFamily) -> Result<(), AdapterError> {
    if mf.get_metric().is_empty() {
        return Err(AdapterError::EmptyMetric);
    }

    if mf.name().is_empty() {
        return Err(AdapterError::UnnamedMetric);
    }

    Ok(())
}

/// Processes [prometheus] histogram buckets and returns a set of [`HistogramBucket`]
fn adapt_hist_buckets(buckets: &[Bucket], sample_count: u64) -> Vec<HistogramBucket> {
    let mut vec = Vec::with_capacity(buckets.len());

    let mut inf_seen = false;
    for b in buckets {
        let upper_bound = b.upper_bound();
        let count = b.cumulative_count();

        let bound = match upper_bound.is_finite() {
            true => HistogramBucketBound::Finite(upper_bound),
            false => HistogramBucketBound::Inf,
        };

        vec.push(HistogramBucket {
            upper_bound: bound,
            count,
        });

        if upper_bound.is_sign_positive() && upper_bound.is_infinite() {
            inf_seen = true;
        }
    }

    if !inf_seen {
        vec.push(HistogramBucket {
            upper_bound: HistogramBucketBound::Inf,
            count: sample_count,
        });
    }

    vec
}

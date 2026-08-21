use std::io::Write;

use prometheus::{
    Encoder, Error,
    proto::{MetricFamily, MetricType},
};

use crate::json_models::{
    HistogramBucketJson, HistogramUpperBound, LabelPariJson, MetricFamilyJson, MetricJson,
    SummaryQuantileJson,
};

/// The json format of metrics
const JSON_ENCODER_FORMAT: &str = "application/json";

/// An implementation of an [`Encoder`] / `Adapter` that converts a [`MetricFamily`] proto
/// message into custom formatted json format
#[derive(Debug, Default)]
pub struct JsonEncoder;

impl JsonEncoder {
    /// Creates a new [`JsonEncoder`]
    pub fn new() -> Self {
        JsonEncoder
    }

    pub fn encode_struct(&self, mfs: &[MetricFamily]) -> prometheus::Result<Vec<MetricFamilyJson>> {
        let mut items = Vec::with_capacity(mfs.len());

        for mf in mfs {
            // Fail-fast checks.
            if mf.get_metric().is_empty() {
                return Err(Error::Msg(format!("MetricFamily has no metrics: {:?}", mf)));
            }
            if mf.name().is_empty() {
                return Err(Error::Msg(format!("MetricFamily has no name: {:?}", mf)));
            };

            // Write metric metadata
            let name = mf.name();
            let help = mf.help();

            let metric_type = mf.get_field_type();
            let lowercase_type = format!("{:?}", metric_type).to_lowercase();

            // Start generating the JSON for the current Metric Family
            let mut family_json = MetricFamilyJson::new(name, help, &lowercase_type);

            // Add metric fields to the generated JSON
            for m in mf.get_metric() {
                match metric_type {
                    MetricType::COUNTER => {
                        let metric = MetricJson::counter(
                            LabelPariJson::from_slice(m.get_label()),
                            m.get_counter().value(),
                        );
                        family_json.add_metric(metric);
                    }
                    MetricType::GAUGE => {
                        let metric = MetricJson::gauge(
                            LabelPariJson::from_slice(m.get_label()),
                            m.get_gauge().value(),
                        );
                        family_json.add_metric(metric);
                    }
                    MetricType::SUMMARY => {
                        let s = m.get_summary();

                        let mut quantiles = Vec::new();
                        for q in s.get_quantile().iter() {
                            quantiles.push(SummaryQuantileJson {
                                quantile: q.quantile(),
                                value: q.value(),
                            });
                        }

                        let sum = s.sample_sum();
                        let count = s.sample_count();

                        let metric = MetricJson::summary(
                            LabelPariJson::from_slice(m.get_label()),
                            quantiles,
                            sum,
                            count,
                        );
                        family_json.add_metric(metric);
                    }
                    MetricType::UNTYPED => {
                        let metric = MetricJson::untyped(
                            LabelPariJson::from_slice(m.get_label()),
                            m.untyped.value(),
                        );
                        family_json.add_metric(metric);
                    }
                    MetricType::HISTOGRAM => {
                        let h = m.get_histogram();

                        let mut buckets = Vec::with_capacity(h.get_bucket().len());

                        let mut inf_seen = false;
                        for b in h.get_bucket() {
                            let upper_bound = b.upper_bound();
                            let count = b.cumulative_count();

                            let bound = match upper_bound.is_finite() {
                                true => HistogramUpperBound::Finite(upper_bound),
                                false => HistogramUpperBound::Inf,
                            };

                            buckets.push(HistogramBucketJson {
                                upper_bound: bound,
                                count,
                            });

                            if upper_bound.is_sign_positive() && upper_bound.is_infinite() {
                                inf_seen = true;
                            }
                        }

                        if !inf_seen {
                            buckets.push(HistogramBucketJson {
                                upper_bound: HistogramUpperBound::Inf,
                                count: h.get_sample_count(),
                            });
                        }

                        let sum = h.get_sample_sum();
                        let count = h.get_sample_count();

                        let metric = MetricJson::histogram(
                            LabelPariJson::from_slice(m.get_label()),
                            buckets,
                            sum,
                            count,
                        );
                        family_json.add_metric(metric);
                    }
                }
            }

            items.push(family_json);
        }
        Ok(items)
    }

    fn encode_impl<W: Write>(
        &self,
        mfs: &[MetricFamily],
        writer: &mut W,
    ) -> prometheus::Result<()> {
        let items = self.encode_struct(mfs)?;
        serde_json::to_writer(writer, &items)
            .map_err(|e| Error::Msg(format!("Failed to write json: {e:?}")))?;

        Ok(())
    }
}

impl Encoder for JsonEncoder {
    fn encode<W: Write>(&self, mfs: &[MetricFamily], writer: &mut W) -> prometheus::Result<()> {
        self.encode_impl(mfs, writer)
    }

    fn format_type(&self) -> &str {
        JSON_ENCODER_FORMAT
    }
}

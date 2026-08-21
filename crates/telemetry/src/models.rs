use serde::Serialize;

/// API representation of a metric
#[derive(Debug, Serialize)]
pub struct MetricApi {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: String,
    /// Metric Family type
    #[serde(rename = "type")]
    pub metric_type: MetricType,
    /// Metric data
    pub data: Vec<MetricData>,
}

impl MetricApi {
    /// Creates a new [`MetricApi`] model
    pub fn new<S, M>(name: S, desciption: S, metric_type: M) -> Self
    where
        S: Into<String>,
        M: Into<MetricType>,
    {
        MetricApi {
            name: name.into(),
            description: desciption.into(),
            metric_type: metric_type.into(),
            data: Vec::new(),
        }
    }

    /// Adds metric data to the model
    pub fn push_metric(&mut self, data: MetricData) {
        self.data.push(data);
    }
}

/// Labeled metric data
#[derive(Debug, Serialize)]
pub struct MetricData {
    /// Labels for the current metric
    labels: Vec<MetricLabel>,

    /// Metric values
    #[serde(flatten)]
    value: MetricValue,
}

impl MetricData {
    /// Creates a new builder for [`MetricData`]
    pub fn builder() -> MetricDataBuilder<false> {
        MetricDataBuilder {
            labels: Vec::new(),
            value: None,
        }
    }
}

/// Builder for [`MetricData`]
pub struct MetricDataBuilder<const WITH_LABEL: bool> {
    labels: Vec<MetricLabel>,
    value: Option<MetricValue>,
}

impl MetricDataBuilder<false> {
    /// Adds labels to the current MetricData
    pub fn with_labels(
        mut self,
        labels: &[prometheus::proto::LabelPair],
    ) -> MetricDataBuilder<true> {
        let mut l = Vec::with_capacity(labels.len());
        for lp in labels {
            let name = lp.name();
            let value = lp.value();

            l.push(MetricLabel {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
        self.labels = l;

        MetricDataBuilder {
            labels: self.labels,
            value: self.value,
        }
    }
}

impl MetricDataBuilder<true> {
    /// Builds a [`MetricData`] as `COUNTER` metric data
    pub fn counter(self, value: f64) -> MetricData {
        MetricData {
            labels: self.labels,
            value: MetricValue::Counter { value },
        }
    }

    /// Builds a [`MetricData`] as `GAUGE` metric data
    pub fn gauge(self, value: f64) -> MetricData {
        MetricData {
            labels: self.labels,
            value: MetricValue::Gauge { value },
        }
    }

    /// Builds a [`MetricData`] as `UNTYPED` metric data
    pub fn untyped(self, value: f64) -> MetricData {
        MetricData {
            labels: self.labels,
            value: MetricValue::Untyped { value },
        }
    }

    /// Builds a [`MetricData`] as `SUMMARY` metric data
    pub fn summary(self, quantiles: Vec<SummaryQuantile>, sum: f64, count: u64) -> MetricData {
        MetricData {
            labels: self.labels,
            value: MetricValue::Summary {
                quantiles,
                sum,
                count,
            },
        }
    }

    /// Builds a [`MetricData`] as `HISTOGRAM` metric data
    pub fn histogram(self, buckets: Vec<HistogramBucket>, sum: f64, count: u64) -> MetricData {
        MetricData {
            labels: self.labels,
            value: MetricValue::Histogram {
                buckets,
                sum,
                count,
            },
        }
    }
}

/// Label group for the metric
#[derive(Debug, Serialize)]
pub struct MetricLabel {
    /// Label name
    pub name: String,
    /// Label value as a string
    pub value: String,
}

/// Enum of various metric type values
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Counter metric value
    Counter {
        /// Counter value
        value: f64,
    },

    /// Gauge metric value
    Gauge {
        /// Gauge value
        value: f64,
    },

    /// Untyped metric value
    Untyped {
        /// Untyped value
        value: f64,
    },

    /// Summary metric value
    Summary {
        /// Summary metric quantiles
        quantiles: Vec<SummaryQuantile>,

        /// The sum of all values ​​recorded in the metric
        sum: f64,

        /// The total number of values ​​recorded in the metric
        count: u64,
    },

    /// Histogram metric value
    Histogram {
        /// Histogram Buckets
        buckets: Vec<HistogramBucket>,

        /// The sum of all values ​​recorded in the metric
        sum: f64,

        /// The total number of values ​​recorded in the metric
        count: u64,
    },
}

/// Quantile of the summary metric
#[derive(Debug, Serialize)]
pub struct SummaryQuantile {
    /// Quantile in [`f64`]
    pub quantile: f64,
    /// Quantile value
    pub value: f64,
}

/// Histogram bucket with boundary specification
#[derive(Debug, Serialize)]
pub struct HistogramBucket {
    /// Upper bound of a histogram bucket
    pub upper_bound: HistogramBucketBound,
    /// Number of records in the bucket
    pub count: u64,
}

/// Histogram bucket boundary
#[derive(Debug, Serialize)]
pub enum HistogramBucketBound {
    /// Equivalent to [`f64::INFINITY`]
    #[serde(rename = "inf")]
    Inf,

    /// Finite [`f64`] value
    #[serde(untagged)]
    Finite(f64),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// [`prometheus::Counter`] metric type
    Counter,
    /// [`prometheus::Gauge`] metric type
    Gauge,
    /// `Summary` metric type
    Summary,
    /// Untyped metric type
    Untyped,
    /// [`prometheus::Histogram`] metric type
    Histogram,
}

impl From<prometheus::proto::MetricType> for MetricType {
    fn from(mt: prometheus::proto::MetricType) -> Self {
        match mt {
            prometheus::proto::MetricType::COUNTER => MetricType::Counter,
            prometheus::proto::MetricType::GAUGE => MetricType::Gauge,
            prometheus::proto::MetricType::SUMMARY => MetricType::Summary,
            prometheus::proto::MetricType::UNTYPED => MetricType::Untyped,
            prometheus::proto::MetricType::HISTOGRAM => MetricType::Histogram,
        }
    }
}

use serde::Serialize;

/// JSON representation of the Metric Family
#[derive(Debug, Serialize)]
pub struct MetricFamilyJson {
    /// Metric Family name
    pub name: String,
    /// Metric Family description
    pub description: String,

    /// Metric Family type
    #[serde(rename = "type")]
    pub metric_type: String,

    /// Metric Family data
    pub data: Vec<MetricJson>,
}

impl MetricFamilyJson {
    /// Creates a new [`MetricFamilyJson`]
    pub fn new(name: &str, description: &str, m_type: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            metric_type: m_type.to_string(),
            data: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, metric: MetricJson) {
        self.data.push(metric);
    }
}

#[derive(Debug, Serialize)]
pub struct MetricJson {
    labels: Vec<LabelPariJson>,

    #[serde(flatten)]
    data: MetricSnapshotJson,
}

impl MetricJson {
    pub fn counter(labels: Vec<LabelPariJson>, value: f64) -> Self {
        MetricJson {
            labels,
            data: MetricSnapshotJson::Counter { value },
        }
    }

    pub fn gauge(labels: Vec<LabelPariJson>, value: f64) -> Self {
        MetricJson {
            labels,
            data: MetricSnapshotJson::Gauge { value },
        }
    }

    pub fn untyped(labels: Vec<LabelPariJson>, value: f64) -> Self {
        MetricJson {
            labels,
            data: MetricSnapshotJson::Untyped { value },
        }
    }

    pub fn summary(
        labels: Vec<LabelPariJson>,
        quantiles: Vec<SummaryQuantileJson>,
        sum: f64,
        count: u64,
    ) -> Self {
        MetricJson {
            labels,
            data: MetricSnapshotJson::Summary {
                quantiles,
                sum,
                count,
            },
        }
    }

    pub fn histogram(
        labels: Vec<LabelPariJson>,
        buckets: Vec<HistogramBucketJson>,
        sum: f64,
        count: u64,
    ) -> Self {
        MetricJson {
            labels,
            data: MetricSnapshotJson::Histogram {
                buckets,
                sum,
                count,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LabelPariJson {
    /// Name of a pair of labels
    pub name: String,
    /// Value of a pair of labels
    pub value: String,
}

impl LabelPariJson {
    pub fn from_slice(slice: &[prometheus::proto::LabelPair]) -> Vec<Self> {
        let mut vec = Vec::with_capacity(slice.len());
        for s in slice {
            let name = s.name();
            let value = s.value();

            vec.push(LabelPariJson {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
        vec
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MetricSnapshotJson {
    Counter {
        value: f64,
    },
    Gauge {
        value: f64,
    },
    Untyped {
        value: f64,
    },
    Summary {
        quantiles: Vec<SummaryQuantileJson>,
        sum: f64,
        count: u64,
    },
    Histogram {
        buckets: Vec<HistogramBucketJson>,
        sum: f64,
        count: u64,
    },
}

#[derive(Debug, Serialize)]
pub struct SummaryQuantileJson {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct HistogramBucketJson {
    pub upper_bound: HistogramUpperBound,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub enum HistogramUpperBound {
    #[serde(rename = "inf")]
    Inf,
    #[serde(untagged)]
    Finite(f64),
}

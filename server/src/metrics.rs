use std::time::Duration;

use telemetry::{HistogramOpts, HistogramVec, MetricsRegistry};

const HTTP_SERVER_DURATION: &str = "http_server_duration";

pub struct ServerMetrics {
    http_server_duration: HistogramVec,
}

impl ServerMetrics {
    pub fn new(reg: &MetricsRegistry) -> Self {
        let http_server_duration = HistogramVec::new(
            HistogramOpts::new(
                HTTP_SERVER_DURATION,
                "Http server requests durations metric",
            ),
            &["method", "route"],
        )
        .expect("");
        reg.register(&http_server_duration).expect("");

        Self {
            http_server_duration,
        }
    }

    pub fn request_finished(&self, elapsed: Duration, method: &str, route: &str) {
        self.http_server_duration
            .with_label_values(&[method, route])
            .observe(elapsed.as_secs_f64());
    }
}

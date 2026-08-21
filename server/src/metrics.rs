use std::time::Duration;

use result::{Result, error::ResultExt};
use telemetry::{HistogramOpts, HistogramVec, MetricsRegistry};

const HTTP_SERVER_DURATION: &str = "http_server_duration_seconds";

/// Server metrics
pub struct ServerMetrics {
    /// HTTP request duration metric (in seconds)
    http_server_duration_seconds: HistogramVec,
}

impl ServerMetrics {
    pub fn try_new(reg: &MetricsRegistry) -> Result<Self> {
        let http_server_duration = HistogramVec::new(
            HistogramOpts::new(
                HTTP_SERVER_DURATION,
                "Http server requests durations metric",
            ),
            &["method", "route"],
        )
        .to_app_err()?;
        reg.register(&http_server_duration).to_app_err()?;

        Ok(Self {
            http_server_duration_seconds: http_server_duration,
        })
    }

    /// Publishes metrics based on the results of an HTTP request execution
    pub fn http_request_finished(&self, elapsed: Duration, method: &str, route: &str) {
        self.http_server_duration_seconds
            .with_label_values(&[method, route])
            .observe(elapsed.as_secs_f64());
    }
}

use std::time::Duration;

use mimetype::MimeKind;
use result::{Result, error::ResultExt};
use telemetry::{CounterVec, HistogramOpts, HistogramVec, MetricsRegistry, Opts};

const SERVER_REQUESTS_DURATION: &str = "server_requests_duration_seconds";
const SERVER_FILES_UPLOADED: &str = "server_files_uploaded";

/// Server metrics
pub struct ServerMetrics {
    /// HTTP request duration metric (in seconds)
    server_requests_duration_seconds: HistogramVec,

    /// Server files uploaded metric
    server_files_uploaded: CounterVec,
}

impl ServerMetrics {
    pub fn try_new(reg: &MetricsRegistry) -> Result<Self> {
        let server_requests_duration_seconds = reg
            .reg_histogram_vec(
                HistogramOpts::new(SERVER_REQUESTS_DURATION, "Server requests durations metric"),
                &["method", "route"],
            )
            .to_app_err()?;

        let server_files_uploaded = reg
            .reg_counter_vec(
                Opts::new(SERVER_FILES_UPLOADED, "Server files uploaded metric"),
                &["type"],
            )
            .to_app_err()?;

        Ok(Self {
            server_requests_duration_seconds,
            server_files_uploaded,
        })
    }

    /// Publishes metrics based on the results of an HTTP request execution
    pub fn http_request_finished(&self, elapsed: Duration, method: &str, route: &str) {
        self.server_requests_duration_seconds
            .with_label_values(&[method, route])
            .observe(elapsed.as_secs_f64());
    }

    /// Publishes file upload metrics
    pub fn file_uploaded(&self, media_type: &MimeKind) {
        self.server_files_uploaded
            .with_label_values(&[media_type.as_str()])
            .inc();
    }
}

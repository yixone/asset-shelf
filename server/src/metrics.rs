use std::time::Duration;

use mimetype::MimeKind;
use result::Result;
use telemetry::{CounterVec, HistogramOpts, HistogramVec, MetricsRegistry, Opts};

const SERVER_REQUESTS_DURATION: &str = "server_requests_duration_seconds";
const SERVER_REQUESTS_TOTAL: &str = "server_requests_total";

const SERVER_FILES_UPLOADED: &str = "server_files_uploaded";

/// Server metrics
pub struct ServerMetrics {
    /// HTTP request duration metric (in seconds)
    server_requests_duration_seconds: HistogramVec,

    /// General statistics on server requests
    server_requests_total: CounterVec,

    /// Server files uploaded metric
    server_files_uploaded: CounterVec,
}

impl ServerMetrics {
    pub fn try_new(reg: &MetricsRegistry) -> Result<Self> {
        Ok(Self {
            server_requests_duration_seconds: reg.reg_histogram_vec(
                HistogramOpts::new(SERVER_REQUESTS_DURATION, "Server requests durations metric"),
                &["method", "route"],
            )?,
            server_requests_total: reg.reg_counter_vec(
                Opts::new(SERVER_REQUESTS_TOTAL, "Total server requests metric"),
                &["method", "route", "status"],
            )?,
            server_files_uploaded: reg.reg_counter_vec(
                Opts::new(SERVER_FILES_UPLOADED, "Server files uploaded metric"),
                &["type"],
            )?,
        })
    }

    /// Publishes metrics based on the results of an HTTP request execution
    pub fn http_request_finished(&self, elapsed: Duration, method: &str, route: &str, status: u16) {
        self.server_requests_duration_seconds
            .observe(elapsed.as_secs_f64(), &[method, route]);

        self.server_requests_total
            .inc(&[method, route, &status.to_string()]);
    }

    /// Publishes file upload metrics
    pub fn file_uploaded(&self, media_type: &MimeKind) {
        self.server_files_uploaded.inc(&[media_type.as_str()]);
    }
}

use serde::{Deserialize, Serialize};

const DEFAULT_VIDEO_SUPPORT_ENABLED: bool = true;
const DEFAULT_METRICS_ENABLED: bool = false;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct InstanceConfig {
    video_support_enabled: bool,
    metrics_enabled: bool,
}

impl InstanceConfig {
    pub fn allow_video(&self) -> bool {
        self.video_support_enabled
    }

    pub fn allow_metrics(&self) -> bool {
        self.metrics_enabled
    }
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            video_support_enabled: DEFAULT_VIDEO_SUPPORT_ENABLED,
            metrics_enabled: DEFAULT_METRICS_ENABLED,
        }
    }
}

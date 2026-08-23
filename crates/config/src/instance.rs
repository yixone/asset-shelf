use serde::{Deserialize, Serialize};

const DEFAULT_INSTANCE_NODE_ID: u8 = 1;
const DEFAULT_VIDEO_SUPPORT_ENABLED: bool = true;
const DEFAULT_METRICS_ENABLED: bool = false;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct InstanceConfig {
    node_id: u8,

    video_support_enabled: bool,
    metrics_enabled: bool,
}

impl InstanceConfig {
    /// Returns the node id of this [`InstanceConfig`]
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

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
            node_id: DEFAULT_INSTANCE_NODE_ID,
            video_support_enabled: DEFAULT_VIDEO_SUPPORT_ENABLED,
            metrics_enabled: DEFAULT_METRICS_ENABLED,
        }
    }
}

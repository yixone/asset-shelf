use serde::{Deserialize, Serialize};

const DEFAULT_INSTANCE_NODE_ID: u8 = 1;
const DEFAULT_VIDEO_SUPPORT: bool = true;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct InstanceConfig {
    node_id: u8,
    video_support: bool,
}

impl InstanceConfig {
    /// Returns the node id of this [`InstanceConfig`]
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    pub fn allow_video(&self) -> bool {
        self.video_support
    }
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            node_id: DEFAULT_INSTANCE_NODE_ID,
            video_support: DEFAULT_VIDEO_SUPPORT,
        }
    }
}

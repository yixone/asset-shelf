use serde::{Deserialize, Serialize};

const DEFAULT_INSTANCE_NODE_ID: u8 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct InstanceConfig {
    node_id: u8,
}

impl InstanceConfig {
    /// Returns the node id of this [`InstanceConfig`]
    pub fn node_id(&self) -> u8 {
        self.node_id
    }
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            node_id: DEFAULT_INSTANCE_NODE_ID,
        }
    }
}

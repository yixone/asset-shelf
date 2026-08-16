use serde::{Deserialize, Serialize};

const DEFAULT_HOST_ADDR: &str = "0.0.0.0:8080";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HostConfig {
    listen_addr: String,
}

impl HostConfig {
    /// Returns a reference to the listen addr of this [`HostConfig`]
    pub fn listen_addr(&self) -> &str {
        &self.listen_addr
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        HostConfig {
            listen_addr: DEFAULT_HOST_ADDR.into(),
        }
    }
}

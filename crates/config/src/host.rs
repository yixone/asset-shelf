use serde::{Deserialize, Serialize};

const DEFAULT_LISTEN_PORT: u16 = 8080;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HostConfig {
    listen_port: u16,
}

impl HostConfig {
    /// Returns a reference to the listen addr of this [`HostConfig`]
    pub fn listen_addr(&self) -> String {
        format!("0.0.0.0:{}", self.listen_port)
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        HostConfig {
            listen_port: DEFAULT_LISTEN_PORT,
        }
    }
}

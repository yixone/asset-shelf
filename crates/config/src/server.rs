use serde::{Deserialize, Serialize};

const DEFAULT_LISTEN_PORT: u16 = 8080;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    /// The port the server is listening on
    listen_port: u16,
}

impl ServerConfig {
    /// Returns a reference to the listen addr of this [`HostConfig`]
    pub fn listen_addr(&self) -> String {
        format!("0.0.0.0:{}", self.listen_port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            listen_port: DEFAULT_LISTEN_PORT,
        }
    }
}

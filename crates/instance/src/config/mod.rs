use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Global application configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Path to the selected library
    ///
    /// If [`None`] - only the instance control panel will be available on the server
    selected_lib_path: Option<String>,

    /// If `true`, the application will collect local telemetry
    telemetry_enabled: bool,

    /// If `true`, allows video downloading and uses ffmpeg to process it
    video_enabled: bool,

    /// Maximum file size that can be uploaded to instance storage (in MB)
    max_file_size_mb: usize,

    /// Server listen port
    listen_port: u16,
}

impl AppConfig {
    /// Returns the `selected_lib_path` of this [`AppConfig`]
    pub fn selected_lib_path(&self) -> Option<PathBuf> {
        self.selected_lib_path.as_ref().map(PathBuf::from)
    }

    /// Sets the selected lib path of this [`AppConfig`]
    pub fn set_selected_lib_path(&mut self, selected_lib_path: String) {
        self.selected_lib_path = Some(selected_lib_path);
    }

    /// Returns the `telemetry_enabled` of this [`AppConfig`]
    pub fn telemetry_enabled(&self) -> bool {
        self.telemetry_enabled
    }

    /// Returns the `max_file_size` of this [`AppConfig`] in bytes
    pub fn max_file_size_bytes(&self) -> usize {
        self.max_file_size_mb * 1024 * 1024
    }

    /// Returns the video enabled of this [`AppConfig`]
    pub fn video_enabled(&self) -> bool {
        self.video_enabled
    }

    /// Returns the listen port of this [`AppConfig`]
    pub fn listen_port(&self) -> u16 {
        self.listen_port
    }

    /// Serializes the [`AppConfig`] as a toml file
    fn serialize(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(&self).map_err(|_| ConfigError::SerializationFailed)
    }

    /// Loads the [`AppConfig`] from the specified file, otherwise creates a new default one
    pub fn load(p: impl AsRef<Path>) -> Result<Self, ConfigError> {
        match std::fs::read(&p) {
            Ok(b) => {
                let deserialized = toml::from_slice::<AppConfig>(&b)
                    .map_err(|_| ConfigError::DeserializationFailed)?;

                Ok(deserialized)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let default = AppConfig::default();
                let str = default.serialize()?;

                let mut file = File::create(&p).map_err(ConfigError::Io)?;
                file.write_all(str.as_bytes()).map_err(ConfigError::Io)?;

                Ok(default)
            }
            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    pub fn write(&self, p: impl AsRef<Path>) -> Result<(), ConfigError> {
        let str = self.serialize()?;

        let mut file = File::create(&p).map_err(ConfigError::Io)?;
        file.write_all(str.as_bytes()).map_err(ConfigError::Io)?;

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            selected_lib_path: None,
            telemetry_enabled: false,
            video_enabled: true,
            max_file_size_mb: 512,
            listen_port: 8080,
        }
    }
}

/// Application configuration error
#[derive(Debug)]
pub enum ConfigError {
    /// Config deserialization error
    DeserializationFailed,

    /// Config serialization error
    SerializationFailed,

    Io(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ConfigError {}

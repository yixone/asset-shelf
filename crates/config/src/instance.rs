use serde::{Deserialize, Serialize};

const DEFAULT_INSTANCE_NODE_ID: u8 = 1;

const DEFAULT_VIDEO_ENABLED: bool = true;
const DEFAULT_SIMILARITY_SEARCH_ENABLED: bool = true;

const DEFAULT_TELEMETRY_ENABLED: bool = false;

/// Application Instance Configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct InstanceConfig {
    /// Instance node id
    node_id: u8,

    /// Instance features config
    pub features: FeaturesConfig,

    /// Instance telemetry config
    pub telemetry: TelemetryConfig,
}

impl InstanceConfig {
    /// Returns the node id of this [`InstanceConfig`]
    pub fn node_id(&self) -> u8 {
        self.node_id
    }
}

/// Application features configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FeaturesConfig {
    /// If `true`, the application will be able to work with video using ffmpeg
    video_enabled: bool,
    /// If `true`, the search function for similar assets will be available
    similarity_search_enabled: bool,
}

impl FeaturesConfig {
    /// If `true`, the application will be able to work with video using ffmpeg
    pub fn video_enabled(&self) -> bool {
        self.video_enabled
    }

    /// If `true`, the search function for similar assets will be available
    pub fn similar_search_enabled(&self) -> bool {
        self.similarity_search_enabled
    }
}

/// Telemetry configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// If `true`, the application will collect local telemetry data
    enabled: bool,
}

impl TelemetryConfig {
    /// If `true`, the application will collect local telemetry data
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        FeaturesConfig {
            video_enabled: DEFAULT_VIDEO_ENABLED,
            similarity_search_enabled: DEFAULT_SIMILARITY_SEARCH_ENABLED,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            enabled: DEFAULT_TELEMETRY_ENABLED,
        }
    }
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            node_id: DEFAULT_INSTANCE_NODE_ID,
            features: FeaturesConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

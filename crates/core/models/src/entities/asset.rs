use chrono::{DateTime, Utc};

use crate::types::{AssetId, MediaId};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Asset {
    pub id: AssetId,
    pub state: AssetState,

    pub media_id: MediaId,

    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub title: Option<String>,
    pub caption: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetState {
    Pending,
    Processing,
    Ready,
    Failed,
}

use chrono::{DateTime, Utc};

use crate::types::MediaId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Media {
    pub id: MediaId,
    pub created_at: DateTime<Utc>,
}

impl Media {
    /// Creates a new [`Media`]
    pub fn new(id: MediaId) -> Self {
        Media {
            id,
            created_at: Utc::now(),
        }
    }
}

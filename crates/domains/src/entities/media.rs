use chrono::{DateTime, Utc};

use crate::MediaId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Media {
    pub id: MediaId,
    pub created_at: DateTime<Utc>,
}

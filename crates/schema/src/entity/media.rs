use chrono::{DateTime, Utc};

use crate::id::MediaId;

/// Media object managed by the application
///
/// A media object groups files containing the same media content,
/// allowing different variants to be associated
/// with a single logical object
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct Media {
    /// Unique string media identifier
    pub id: MediaId,

    /// Media creation time
    pub created_at: DateTime<Utc>,
}

impl Media {
    /// Creates a new [`Media`]
    pub fn new(id: MediaId) -> Self {
        Self {
            id,
            created_at: Utc::now(),
        }
    }
}

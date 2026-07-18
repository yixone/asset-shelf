use chrono::{DateTime, Utc};

use crate::CollectionId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Collection {
    pub id: CollectionId,

    pub name: String,
    pub description: Option<String>,

    pub created_at: DateTime<Utc>,
}

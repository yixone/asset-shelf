use chrono::{DateTime, Duration, Utc};
use mimetype::MimeKind;

use crate::{
    assets::{AssetFeatures, AssetState},
    types::{AssetId, MediaId},
};

/// An asset domain representing a media file in storage
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Asset {
    /// Unique numeric asset ID
    pub id: AssetId,
    /// Asset state
    pub state: AssetState,

    /// Related media group identifier
    pub media_id: MediaId,
    /// Generalized asset media type
    pub media_type: MimeKind,

    /// Asset creation date
    pub created_at: DateTime<Utc>,
    /// Asset update date
    pub updated_at: DateTime<Utc>,
    /// Asset deletion date
    pub deleted_at: Option<DateTime<Utc>>,

    /// Asset title
    pub title: Option<String>,
    /// Optional asset description
    pub caption: Option<String>,
    /// URL from which the asset was obtained
    pub source_url: Option<String>,
}

impl Asset {
    /// Creates a new [`Asset`]
    pub fn new(
        id: AssetId,
        media_id: MediaId,
        media_type: MimeKind,
        title: Option<String>,
        caption: Option<String>,
        source_url: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            state: AssetState::Pending,
            media_id,
            media_type,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            title,
            caption,
            source_url,
        }
    }

    /// Returns `true` if the current [`Asset`] is marked as deletet
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// The time that must elapse before another attempt
    /// can be made to process the asset following a failed attempt
    pub const TIME_BEFORE_REPROCESSING: Duration = Duration::minutes(5);

    /// Checks whether the [`Asset`] requires processing
    pub fn need_processing(&self, feats: &AssetFeatures, now: DateTime<Utc>) -> bool {
        let state = self.state;

        // The asset has not yet been processed
        if state == AssetState::Pending {
            return true;
        }

        // The asset was not processed due to an error or hang
        let need_failed_check = matches!(state, AssetState::Processing | AssetState::Failed);

        if need_failed_check && ((now - self.updated_at).num_minutes() >= 5) {
            return true;
        }

        // The asset was processed previously but currently lacks all the necessary fields
        if state == AssetState::Ready && !feats.enough_fields() {
            return true;
        }

        false
    }

    /// Returns `true` if the current [`Asset`] is an image
    pub fn is_image(&self) -> bool {
        matches!(self.media_type, MimeKind::Image)
    }

    /// Returns `true` if the current [`Asset`] is a video
    pub fn is_video(&self) -> bool {
        matches!(self.media_type, MimeKind::Video)
    }
}

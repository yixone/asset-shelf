use std::{path::Path, time::Duration};

use media::{
    image::Image,
    video::{
        self, ExtractVideoFragmentParams, FragmentParams, ResizeParams,
        input::MediaInput,
        types::{AudioMode, VideoMetadata},
    },
};
use mimetype::MimeType;
use models::media::MediaVariant;
use result::{Result, create_error, error::ResultExt};
use storage::files::ReservedFile;
use tokio::time::timeout;

use crate::media::{
    extracted::{ExtractedFeatures, GeneratedImageVariant, GeneratedVideoVariant},
    processing::image::ImageProcessor,
};

const PROBE_TIMEOUT: Duration = Duration::from_mins(10);
const EXTRACT_FRAME_TIMEOUT: Duration = Duration::from_mins(2);
const TRANSCODINIG_TIMEOUT: Duration = Duration::from_mins(5);

const LOOP_PREVIEW_FRAGMENT_DURATION_MS: u64 = 5000;
const LOOP_PREVIEW_WIDTH: u32 = 600;

/// Video processor entity
pub struct VideoProcessor {
    video: MediaInput,
    meta: VideoMetadata,
}

impl VideoProcessor {
    pub async fn open_video(path: &Path) -> Result<Self> {
        let input = video::input::MediaInput::try_new(path).to_app_err()?;

        let metadata = match timeout(PROBE_TIMEOUT, media::video::probe_video(&input)).await {
            Ok(p) => p.to_app_err()?,
            Err(_) => return Err(create_error!(ProcessingTimeout)),
        };

        Ok(VideoProcessor {
            video: input,
            meta: metadata,
        })
    }

    async fn extract_frame(&self, time_secs: u64) -> Result<Image> {
        let frame = match timeout(
            EXTRACT_FRAME_TIMEOUT * (self.meta.video.duration_secs.round() as u32).max(1),
            media::video::extract_frame(Duration::from_secs(time_secs), &self.video),
        )
        .await
        {
            Ok(f) => {
                let f = f.to_app_err()?;
                Image::from_dynamic(f)
            }
            Err(_) => {
                return Err(create_error!(ProcessingTimeout));
            }
        };

        Ok(frame)
    }

    pub async fn generate_thumbnail(&self) -> Result<GeneratedImageVariant> {
        let frame = self.extract_frame(0).await?;
        ImageProcessor::new(frame).generate_thumbnail()
    }

    pub async fn generate_loop_preview<'a>(
        &self,
        write_to: ReservedFile<'a>,
    ) -> Result<GeneratedVideoVariant<'a>> {
        let duration_ms = (self.meta.video.duration_secs * 1000.0).round() as u64;

        let fragment_duration_ms = LOOP_PREVIEW_FRAGMENT_DURATION_MS.min(duration_ms);
        let fragment_duration = Duration::from_millis(fragment_duration_ms);

        match timeout(
            TRANSCODINIG_TIMEOUT * (duration_ms / 1000).max(1) as u32,
            video::extract_video_fragment(
                &self.video,
                write_to.path(),
                ExtractVideoFragmentParams {
                    fragment: FragmentParams {
                        start: Duration::from_millis(0),
                        duration: fragment_duration,
                    },
                    frame_rate: None,
                    audio: AudioMode::Disabled,
                    output_resolution: ResizeParams::ForceWidth {
                        w: LOOP_PREVIEW_WIDTH,
                    },
                },
            ),
        )
        .await
        {
            Ok(f) => {
                f.to_app_err()?;
            }
            Err(_) => {
                return Err(create_error!(ProcessingTimeout));
            }
        }

        Ok(GeneratedVideoVariant {
            variant: MediaVariant::LoopPreview,
            mimetype: MimeType::Mp4,
            duration_milis: fragment_duration.as_millis() as u64,
            reserve: write_to,
        })
    }

    pub async fn extract_features(&self) -> Result<ExtractedFeatures> {
        let mid = self.meta.video.duration_secs.round() as u64 / 2;
        let frame = self.extract_frame(mid).await?;
        Ok(ImageProcessor::new(frame).extract_features())
    }

    /// Returns a reference to the metadata of this [`VideoPipeline`]
    pub fn metadata(&self) -> &VideoMetadata {
        &self.meta
    }
}

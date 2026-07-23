use actix_multipart::Multipart;
use actix_web::{HttpResponse, post, web};
use chrono::Utc;
use db::{
    core::provider::{DatabaseConnector, TransactionUnit},
    ops::{AssetOps, MediaFilesOps, MediaOps},
};
use futures::TryStreamExt;
use models::entities::{Asset, AssetState, Media, MediaFile, MediaVariant};
use result::{create_error, error::ResultExt};

use crate::{
    di::DataCtx,
    dto::v1::assets::AssetDtoV1,
    routes::ApiResult,
    utils::multipart::{FieldExt, MultipartParseError},
};

const DEFAULT_VARIANT: MediaVariant = MediaVariant::Original;

/// Uploads an asset with a media file
#[post("/upload")]
pub async fn upload_asset(mut payload: Multipart, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut upload = UploadingContext::default();

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|_| MultipartParseError::ReadError)
        .to_app_err()?
    {
        let Some(field_name) = field.name() else {
            continue;
        };

        match field_name {
            "file" => {
                if upload.media.is_some() {
                    continue;
                }

                let now = Utc::now();
                let media = Media {
                    id: ctx.flake.generate_as(),
                    created_at: now,
                };

                // TODO!: Fix storage to avoid accumulating junk caused by errors
                // E.g use `TempStorageFile` as temp file identifier
                let put_result = ctx
                    .storage
                    .upload(DEFAULT_VARIANT.as_str(), field.into_async_reader())
                    .await?;

                let media_file = MediaFile {
                    id: ctx.flake.generate_as(),
                    media_id: media.id.clone(),
                    variant: DEFAULT_VARIANT,
                    storage_path: put_result.path.to_string(),
                    created_at: now,
                    size_bytes: put_result.size_bytes as i64,
                    mimetype: put_result.mimetype,
                };

                let mut tx = ctx.db.begin().await?;
                tx.insert_media(&media).await?;
                tx.insert_media_file(&media_file).await?;
                tx.commit().await?;

                upload.media = Some((media, media_file));
            }
            "title" => upload.title = Some(field.read_to_string().await?),
            "caption" => upload.caption = Some(field.read_to_string().await?),
            "source_url" => upload.source_url = Some(field.read_to_string().await?),
            _ => continue,
        }
    }

    let Some((media, media_file)) = upload.media else {
        return Err(create_error!(MalformedPayload));
    };

    let asset = Asset {
        id: ctx.flake.generate_as(),
        state: AssetState::Pending,
        media_id: media.id.clone(),
        created_at: Utc::now(),
        deleted_at: None,
        title: upload.title,
        caption: upload.caption,
        source_url: upload.source_url,
    };
    {
        let mut conn = ctx.db.acquire().await?;
        conn.insert_asset(&asset).await?;
    }

    let res = AssetDtoV1::from((asset, (media, vec![media_file])));
    Ok(HttpResponse::Created().json(res))
}

/// Asset uploading context
#[derive(Default)]
struct UploadingContext {
    media: Option<(Media, MediaFile)>,

    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,
}

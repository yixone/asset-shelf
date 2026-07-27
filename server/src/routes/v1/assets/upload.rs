use actix_multipart::Multipart;
use actix_web::{HttpResponse, post, web};
use chrono::Utc;
use db::{
    database::{DatabaseProvider, DatabaseTransaction},
    ops::{AssetFeaturesOps, AssetOps, MediaFilesOps, MediaOps},
};
use futures::TryStreamExt;
use models::entities::{Asset, AssetFeatures, AssetState, Media, MediaFile, MediaVariant};
use result::{create_error, error::ResultExt};
use storage::file::UnreleasedFile;
use workers::events::event::AssetCreatedEvent;

use crate::{
    di::DataCtx,
    dto::v1::assets::AssetDtoV1,
    routes::ApiResult,
    utils::multipart::{FieldExt, MultipartParseError},
};

const DEFAULT_VARIANT: MediaVariant = MediaVariant::Original;

/// Asset uploading context
#[derive(Default)]
struct UploadingContext<'a> {
    file: Option<UnreleasedFile<'a>>,

    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,
}

/// Uploads an asset with a media file
#[post("/upload")]
async fn upload_asset(mut payload: Multipart, ctx: web::Data<DataCtx>) -> ApiResult {
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
                if upload.file.is_some() {
                    continue;
                }
                let stream = field.into_async_reader();
                let temp_stored = ctx
                    .storage
                    .upload(MediaVariant::Original.as_str(), stream)
                    .await?;
                upload.file = Some(temp_stored);
            }
            "title" => upload.title = Some(field.read_to_string().await?),
            "caption" => upload.caption = Some(field.read_to_string().await?),
            "source_url" => upload.source_url = Some(field.read_to_string().await?),
            _ => continue,
        }
    }

    let Some(file) = upload.file else {
        return Err(create_error!(MalformedPayload));
    };

    let now = Utc::now();
    let media = Media {
        id: ctx.flake.get_id_as(),
        created_at: now,
    };
    let media_file = MediaFile {
        id: ctx.flake.get_id_as(),
        media_id: media.id.clone(),
        variant: DEFAULT_VARIANT,
        storage_path: file.target.path.to_string(),
        created_at: now,
        size_bytes: file.target.size_bytes as i64,
        mimetype: file.target.mimetype,
    };
    let asset = Asset {
        id: ctx.flake.get_id_as(),
        state: AssetState::Pending,
        media_id: media.id.clone(),
        media_type: media_file.mimetype.kind(),
        created_at: now,
        deleted_at: None,
        title: upload.title,
        caption: upload.caption,
        source_url: upload.source_url,
    };
    let asset_features = AssetFeatures {
        asset_id: asset.id,
        p_hash: None,
        a_hash: None,
        width: None,
        height: None,
        accent_color: None,
    };

    let mut tx = ctx.db.begin().await?;

    tx.insert_media(&media).await?;
    tx.insert_asset(&asset).await?;
    tx.insert_asset_features(&asset_features).await?;
    tx.insert_media_file(&media_file).await?;

    let file = file.release().await?;
    if let Err(e) = tx.commit().await {
        ctx.storage.remove_safely(&file.path).await;
        return Err(e);
    }

    ctx.events.publish(AssetCreatedEvent { asset: asset.id });

    let res = AssetDtoV1::from((asset, asset_features, vec![media_file]));
    Ok(HttpResponse::Created().json(res))
}

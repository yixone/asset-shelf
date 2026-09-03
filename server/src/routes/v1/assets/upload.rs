use actix_multipart::Multipart;
use actix_web::{HttpResponse, post, web};
use events::AssetCreatedEvent;
use futures::TryStreamExt;
use jobs::Job;
use mimetype::MimeType;
use models::{
    assets::{Asset, AssetFeatures},
    media::{Media, MediaFile, MediaVariant},
};
use result::{create_error, error::ResultExt};
use storage::{files::UncommitedFile, global::GlobalPathData};

use crate::{
    di::{DataCtx, MetricsCtx},
    dto::v1::assets::AssetDtoV1,
    routes::ApiResult,
    utils::multipart::{FieldExt, MultipartParseError},
};

const DEFAULT_VARIANT: MediaVariant = MediaVariant::Original;

/// Asset uploading context
#[derive(Default)]
struct UploadingContext<'a> {
    upload: Option<UploadFile<'a>>,

    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,
}

struct UploadFile<'a> {
    file: UncommitedFile<'a>,
    mimetype: MimeType,
}

/// Uploads an asset with a media file
#[post("/upload")]
async fn upload_asset(
    mut payload: Multipart,
    ctx: web::Data<DataCtx>,
    metrics: web::Data<MetricsCtx>,
) -> ApiResult {
    let mut upload = UploadingContext::default();

    let media = Media::new(ctx.flake.get_id_as());

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
                if upload.upload.is_some() {
                    continue;
                }
                let stream = field.into_async_reader();

                let mut size_bytes = 0;
                let mut magic_bytes_buffer = Vec::with_capacity(128);

                let temp_stored = ctx
                    .storage
                    .upload(
                        GlobalPathData::new(&media.id.to_string(), DEFAULT_VARIANT.as_str()),
                        stream,
                        |chunk| {
                            size_bytes += chunk.len();

                            if size_bytes > ctx.config.storage.max_size_bytes() {
                                return Err(create_error!(FileTooLarge {
                                    received: size_bytes,
                                    max_size: ctx.config.storage.max_size_bytes()
                                }));
                            }

                            let header_len = magic_bytes_buffer.len();
                            if header_len < magic_bytes_buffer.capacity() {
                                let h_chunk = &chunk
                                    [..chunk.len().min(magic_bytes_buffer.capacity() - header_len)];
                                magic_bytes_buffer.extend_from_slice(h_chunk);
                            }

                            Ok(())
                        },
                    )
                    .await?;

                let mimetype = match MimeType::guess(&magic_bytes_buffer) {
                    Ok(m) => m,
                    Err(_) => {
                        return Err(create_error!(UnsupportedFileType));
                    }
                };

                if mimetype.is_video() && !ctx.config.instance.features.video_enabled() {
                    return Err(create_error!(FeatureDisabled { feature: "video" }));
                }

                upload.upload = Some(UploadFile {
                    file: temp_stored,
                    mimetype,
                });
            }
            "title" => upload.title = Some(field.read_to_string().await?),
            "caption" => upload.caption = Some(field.read_to_string().await?),
            "source_url" => upload.source_url = Some(field.read_to_string().await?),
            _ => continue,
        }
    }

    let Some(file) = upload.upload else {
        return Err(create_error!(MalformedPayload));
    };

    let media_file = MediaFile::new(
        ctx.flake.get_id_as(),
        media.id.clone(),
        DEFAULT_VARIANT,
        file.file.global_path().to_string(),
        file.file.size_bytes as i64,
        file.mimetype,
        None,
    );
    let asset = Asset::new(
        ctx.flake.get_id_as(),
        media.id.clone(),
        media_file.mimetype.kind(),
        upload.title,
        upload.caption,
        upload.source_url,
    );
    let asset_features = AssetFeatures::new(asset.id);

    let mut op = ctx.db.assets.create_op().await?;

    op.insert_media(&media).await?;
    op.insert_media_file(&media_file).await?;
    op.insert_asset(&asset).await?;
    op.insert_features(&asset_features).await?;

    let file = file.file.commit().await?;

    if let Err(e) = op.commit().await {
        ctx.storage.remove_safely(&file.global_path).await;
        return Err(e);
    }

    metrics.server.file_uploaded(&media_file.mimetype.kind());
    ctx.events.publish(AssetCreatedEvent { asset_id: asset.id });
    ctx.jobs.enqueue(Job::ProcessAssetMedia { id: asset.id });

    let res = AssetDtoV1::from((asset, asset_features, vec![media_file]));
    Ok(HttpResponse::Created().json(res))
}

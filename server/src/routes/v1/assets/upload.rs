use actix_multipart::Multipart;
use actix_web::{HttpResponse, post, web};
use chrono::Utc;
use events::AssetCreatedEvent;
use futures::TryStreamExt;
use mimetype::MimeType;
use models::entities::{Asset, AssetFeatures, AssetState, Media, MediaFile, MediaVariant};
use result::{create_error, error::ResultExt};
use storage::{files::UncommitedFile, global::GlobalPathData};

use crate::{
    di::DataCtx,
    dto::v1::assets::AssetDtoV1,
    routes::ApiResult,
    utils::multipart::{FieldExt, MultipartParseError},
};

const MAX_SIZE: usize = 1024 * 1024 * 1024;

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
async fn upload_asset(mut payload: Multipart, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut upload = UploadingContext::default();

    let now = Utc::now();
    let media = Media {
        id: ctx.flake.get_id_as(),
        created_at: now,
    };

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

                            if size_bytes > MAX_SIZE {
                                return Err(create_error!(FileTooLarge {
                                    received: size_bytes,
                                    max_size: MAX_SIZE
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

    let media_file = MediaFile {
        id: ctx.flake.get_id_as(),
        media_id: media.id.clone(),
        variant: DEFAULT_VARIANT,
        storage_path: file.file.global_path().to_string(),
        created_at: now,
        size_bytes: file.file.size_bytes as i64,
        mimetype: file.mimetype,
        duration_milis: None,
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

    ctx.db.media.insert(&media).await?;
    ctx.db.media.insert_file(&media_file).await?;
    ctx.db.assets.insert(&asset, &asset_features).await?;

    file.file.commit().await?;

    // if let Err(e) = tx.commit().await {
    //     ctx.storage.remove_safely(&file.global_path).await;
    //     return Err(e);
    // }

    ctx.events.publish(AssetCreatedEvent { asset: asset.id });

    let res = AssetDtoV1::from((asset, asset_features, vec![media_file]));
    Ok(HttpResponse::Created().json(res))
}

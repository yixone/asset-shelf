use std::str::FromStr;

use actix_web::{
    HttpRequest, HttpResponse, get,
    http::header::{self, ACCEPT_RANGES, ContentLength, ContentRange},
    web,
};

use events::FileDetachedEvent;
use mimetype::MimeKind;
use models::{
    media::{MediaFile, MediaVariant},
    types::MediaId,
};
use result::{ErrorKind, Result, create_error, error::ResultExt};
use storage::StoragePath;
use tokio_util::io::ReaderStream;

use crate::{di::DataCtx, routes::ApiResult, utils};

#[get("/{id}/{variant}")]
async fn get_media_file(
    path: web::Path<(MediaId, MediaVariant)>,
    request: HttpRequest,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let (id, variant) = path.into_inner();

    let media_file = ctx.db.media.get_variant(&id, variant).await?;

    let stream = match media_file.mimetype.kind() {
        MimeKind::Image => return_image_stream(&ctx, &media_file).await?,
        MimeKind::Video => return_video_stream(&ctx, &media_file, &request).await?,
    };
    match stream {
        Some(stream) => Ok(stream),
        None => {
            let path = StoragePath::from_str(&media_file.storage_path).to_app_err()?;
            tracing::error!(
                path = path.to_string(),
                "Storage desynchronization: The file exists in the database but is missing from the storage!"
            );
            ctx.events.publish(FileDetachedEvent { media: id, path });
            Err(create_error!(FileDetached))
        }
    }
}

async fn return_image_stream(
    ctx: &DataCtx,
    media_file: &MediaFile,
) -> Result<Option<HttpResponse>> {
    let mimetype = media_file.mimetype.as_str();
    let content_len = media_file.size_bytes;

    let path = StoragePath::from_str(&media_file.storage_path).to_app_err()?;

    match ctx.storage.open(&path).await {
        Ok(reader) => {
            let stream = ReaderStream::new(reader);

            Ok(Some(
                HttpResponse::Ok()
                    .content_type(mimetype)
                    .append_header(ContentLength(content_len as usize))
                    .streaming(stream),
            ))
        }
        Err(e) if matches!(e.kind(), ErrorKind::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

async fn return_video_stream(
    ctx: &DataCtx,
    media_file: &MediaFile,
    req: &HttpRequest,
) -> Result<Option<HttpResponse>> {
    let mimetype = media_file.mimetype.as_str();
    let content_len = media_file.size_bytes;

    let range = utils::header::parse_request_range(req, content_len as u64);

    let path = StoragePath::from_str(&media_file.storage_path).to_app_err()?;

    let reader = match range {
        Some((from, to)) => ctx.storage.open_ranged(&path, from, Some(to)).await,
        None => ctx.storage.open(&path).await,
    };

    match reader {
        Ok(reader) => {
            let stream = ReaderStream::new(reader);

            match range {
                Some((from, to)) => Ok(Some(
                    HttpResponse::PartialContent()
                        .content_type(mimetype)
                        .append_header(ContentLength((to.saturating_sub(from) + 1) as usize))
                        .append_header(ContentRange(header::ContentRangeSpec::Bytes {
                            range,
                            instance_length: Some(content_len as u64),
                        }))
                        .append_header((ACCEPT_RANGES, "bytes"))
                        .streaming(stream),
                )),
                None => Ok(Some(
                    HttpResponse::Ok()
                        .content_type(mimetype)
                        .append_header(ContentLength(content_len as usize))
                        .append_header((ACCEPT_RANGES, "bytes"))
                        .streaming(stream),
                )),
            }
        }
        Err(e) if matches!(e.kind(), ErrorKind::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

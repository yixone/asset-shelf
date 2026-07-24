use std::str::FromStr;

use actix_web::{HttpResponse, get, http::header::ContentLength, web};
use db::{core::provider::DatabaseConnector, ops::MediaFilesOps};
use models::{entities::MediaVariant, types::MediaId};
use result::{create_error, error::ResultExt};
use serde::Deserialize;
use storage_backend::core::path::StoragePath;
use tokio_util::io::ReaderStream;

use crate::{di::DataCtx, routes::ApiResult};

#[derive(Deserialize)]
struct GetMediaFileQuery {
    #[serde(default)]
    format: MediaVariant,
}

#[get("/{id}")]
async fn get_media_file(
    id: web::Path<MediaId>,
    query: web::Query<GetMediaFileQuery>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let query = query.into_inner();

    let media_file = {
        let mut conn = ctx.db.acquire().await?;
        conn.get_media_variant(&id, query.format)
            .await?
            .ok_or(create_error!(NotFound))?
    };

    let mimetype = media_file.mimetype.as_str();
    let content_len = media_file.size_bytes;

    let path = StoragePath::from_str(&media_file.storage_path).to_app_err()?;

    let reader = ctx.storage.get(&path).await?;
    let stream = ReaderStream::new(reader);

    Ok(HttpResponse::Ok()
        .content_type(mimetype)
        .append_header(ContentLength(content_len as usize))
        .streaming(stream))
}

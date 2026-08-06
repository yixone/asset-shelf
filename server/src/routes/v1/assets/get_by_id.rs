use actix_web::{HttpResponse, get, web};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesReadOps, AssetsReadOps, MediaFilesReadOps},
};
use models::types::AssetId;
use result::create_error;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[get("/{id}")]
async fn get_asset_by_id(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut conn = ctx.db.acquire().await?;

    let asset = conn
        .get_asset_by_id(&id)
        .await?
        .ok_or(create_error!(NotFound))?;
    let feats = conn
        .get_asset_features_by_id(&id)
        .await?
        .ok_or(create_error!(NotFound))?;

    let media = conn.get_media_files_by_group(&asset.media_id).await?;

    let res = AssetDtoV1::from((asset, feats, media));
    Ok(HttpResponse::Ok().json(res))
}

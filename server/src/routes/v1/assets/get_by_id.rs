use actix_web::{HttpResponse, get, web};
use models::types::AssetId;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[get("/{id}")]
async fn get_asset_by_id(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let asset = ctx.db.assets.get_by_id(*id).await?;

    let res = AssetDtoV1::from(asset);
    Ok(HttpResponse::Ok().json(res))
}

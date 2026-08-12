use actix_web::{HttpResponse, post, web};
use db::types::{UpdateResult, patch::AssetPatch};
use models::types::AssetId;
use result::create_error;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[post("/{id}/restore")]
async fn restore(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let patch = AssetPatch::new().deleted_at(None);
    match ctx.db.assets.update(*id, patch).await? {
        UpdateResult::Updated(a) => {
            let res = AssetDtoV1::from(a);

            Ok(HttpResponse::Ok().json(res))
        }
        UpdateResult::NotFound => Err(create_error!(NotFound)),
    }
}

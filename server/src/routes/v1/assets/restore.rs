use actix_web::{HttpResponse, post, web};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesOps, AssetOps, MediaFilesOps},
    types::{UpdateResult, patches::AssetPatch},
};
use models::types::AssetId;
use result::create_error;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[post("/{id}/restore")]
async fn restore(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut conn = ctx.db.acquire().await?;
    let patch = AssetPatch::new().deleted_at(None);
    match conn.update_asset(&id, patch).await? {
        UpdateResult::Updated(a) => {
            let feats = conn
                .get_asset_features(&id)
                .await?
                .ok_or(create_error!(NotFound))?;
            let media = conn.get_media_files(&a.media_id).await?;
            let res = AssetDtoV1::from((a, feats, media));

            Ok(HttpResponse::Ok().json(res))
        }
        UpdateResult::NotFound => Err(create_error!(NotFound)),
    }
}

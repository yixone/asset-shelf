use actix_web::{HttpResponse, post, web};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesReadOps, AssetsWriteOps, MediaFilesReadOps},
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
                .get_asset_features_by_id(&id)
                .await?
                .ok_or(create_error!(NotFound))?;
            let media = conn.get_media_files_by_group(&a.media_id).await?;
            let res = AssetDtoV1::from((a, feats, media));

            Ok(HttpResponse::Ok().json(res))
        }
        UpdateResult::NotFound => Err(create_error!(NotFound)),
    }
}

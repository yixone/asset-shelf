use actix_web::{HttpResponse, delete, web};
use db::{database::DatabaseProvider, ops::CollectionAssetsOps};
use models::types::{CollectionAssetId, CollectionId};

use crate::{di::DataCtx, routes::ApiResult};

#[delete("/{id}/assets/{rel_id}")]
async fn remove_collection_asset(
    ids: web::Path<(CollectionId, CollectionAssetId)>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let (_, relation_id) = ids.into_inner();

    let mut conn = ctx.db.acquire().await?;
    conn.remove_collection_asset(&relation_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

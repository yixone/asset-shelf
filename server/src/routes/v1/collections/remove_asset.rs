use actix_web::{HttpResponse, delete, web};
use models::types::{CollectionAssetId, CollectionId};

use crate::{di::DataCtx, routes::ApiResult};

#[delete("/{id}/assets/{rel_id}")]
async fn remove_collection_asset(
    ids: web::Path<(CollectionId, CollectionAssetId)>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let (collection, relation_id) = ids.into_inner();

    ctx.db
        .collections
        .remove_asset(collection, relation_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

use actix_web::{HttpResponse, get, web};
use models::types::CollectionId;

use crate::{di::DataCtx, dto::v1::collections::CollectionDtoV1, routes::ApiResult};

#[get("/{id}")]
async fn get_collection_by_id(id: web::Path<CollectionId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let collection = ctx.db.collections.get_by_id(*id).await?;
    let res = CollectionDtoV1::from(collection);

    Ok(HttpResponse::Ok().json(res))
}

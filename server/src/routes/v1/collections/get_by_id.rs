use actix_web::{HttpResponse, get, web};
use db::{
    database::DatabaseProvider,
    ops::{CollectionsReadOps, CollectionsRelationsOps},
};
use models::types::CollectionId;
use result::create_error;

use crate::{di::DataCtx, dto::v1::collections::CollectionDtoV1, routes::ApiResult};

#[get("/{id}")]
async fn get_collection_by_id(id: web::Path<CollectionId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut conn = ctx.db.acquire().await?;
    let collection = conn
        .get_collection_by_id(&id)
        .await?
        .ok_or(create_error!(NotFound))?;

    let additions = conn
        .get_collections_additions_by_ids(std::slice::from_ref(&id))
        .await?
        .into_iter()
        .next()
        .expect("CollectionAdditions were not calculated for the existing collection");

    drop(conn);

    let res = CollectionDtoV1::from((collection, additions));
    Ok(HttpResponse::Ok().json(res))
}

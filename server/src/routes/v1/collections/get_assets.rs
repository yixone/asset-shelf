use actix_web::{HttpResponse, get, web};
use db::types::Pagination;
use models::types::{CollectionAssetsOrdering, CollectionId};
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::collections::CollectionAssetDtoV1, routes::ApiResult};

#[derive(Deserialize, Default)]
#[serde(default)]
struct GetCollectionAssetsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ordering: Option<CollectionAssetsOrdering>,
}

#[get("/{id}/assets")]
async fn get_collection_assets(
    id: web::Path<CollectionId>,
    query: web::Query<GetCollectionAssetsQuery>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let q = query.into_inner();
    let pagination = Pagination::try_new(q.limit.unwrap_or(50), q.offset.unwrap_or(0))?;

    let assets = ctx
        .db
        .collections
        .get_items(*id, pagination, q.ordering.unwrap_or_default())
        .await?;

    let res = assets
        .into_iter()
        .map(CollectionAssetDtoV1::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(res))
}

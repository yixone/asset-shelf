use actix_web::{HttpResponse, get, web};
use db::types::Pagination;
use models::types::CollectionsOrdering;
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::collections::CollectionDtoV1, routes::ApiResult};

#[derive(Deserialize, Default)]
#[serde(default)]
struct GetCollectionsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ordering: Option<CollectionsOrdering>,
}

#[get("")]
async fn get_collections_list(
    query: web::Query<GetCollectionsQuery>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let q = query.into_inner();
    let pagination = Pagination::try_new(q.limit.unwrap_or(50), q.offset.unwrap_or(0))?;

    let collections = ctx
        .db
        .collections
        .list(pagination, q.ordering.unwrap_or_default())
        .await?;

    let res = collections
        .into_iter()
        .map(CollectionDtoV1::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(res))
}

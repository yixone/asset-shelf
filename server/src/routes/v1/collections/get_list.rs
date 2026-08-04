use actix_web::{HttpResponse, get, web};
use db::{database::DatabaseProvider, ops::CollectionsOps, types::Pagination};
use join::JoinBuilder;
use models::{bulk::BulkIds, types::CollectionsOrdering};
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

    let mut db = ctx.db.acquire().await?;
    let c = db
        .list_collections(pagination, q.ordering.unwrap_or_default())
        .await?;
    let ca = db.get_collections_additions_bulk(&c.ids()).await?;
    drop(db);

    let res = JoinBuilder::new(c)
        .with(ca, |c| c)
        .transform(|(c, ca)| (c, ca))
        .build_as(CollectionDtoV1::from);

    Ok(HttpResponse::Ok().json(res))
}

use actix_web::{
    HttpResponse, get,
    web::{self},
};
use db::types::Pagination;
use models::types::AssetsOrdering;
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[derive(Deserialize, Default)]
#[serde(default)]
struct GetDeletedAssetsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ordering: Option<AssetsOrdering>,
}

#[get("/deleted")]
async fn get_deleted_assets(
    query: web::Query<GetDeletedAssetsQuery>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let q = query.into_inner();
    let pagination = Pagination::try_new(q.limit.unwrap_or(50), q.offset.unwrap_or(0))?;

    let assets = ctx
        .db
        .assets
        .get_deleted(pagination, q.ordering.unwrap_or_default())
        .await?;

    let res = assets.into_iter().map(AssetDtoV1::from).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(res))
}

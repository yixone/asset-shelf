use actix_web::{
    HttpResponse, get,
    web::{self},
};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesReadOps, AssetsReadOps, MediaFilesReadOps},
    types::Pagination,
};
use join::JoinBuilder;
use models::{bulk::BulkIds, types::AssetsOrdering};
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

#[derive(Deserialize, Default)]
#[serde(default)]
struct GetAssetsListQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ordering: Option<AssetsOrdering>,
}

#[get("")]
async fn get_assets_list(
    query: web::Query<GetAssetsListQuery>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let q = query.into_inner();
    let pagination = Pagination::try_new(q.limit.unwrap_or(50), q.offset.unwrap_or(0))?;

    let mut db = ctx.db.acquire().await?;

    let assets = db
        .list_assets(pagination, q.ordering.unwrap_or_default())
        .await?;

    let feats = db.get_assets_features_by_ids(&assets.ids()).await?;
    let media = db.get_media_files_by_groups(&assets.ids()).await?;

    let res = JoinBuilder::new(assets)
        .with(feats, |a| a)
        .with_group(media, |(a, _)| a)
        .transform(|((a, af), mf)| (a, af, mf))
        .build_as(AssetDtoV1::from);

    Ok(HttpResponse::Ok().json(res))
}

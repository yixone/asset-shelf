use actix_web::{HttpResponse, get, web};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesReadOps, AssetsReadOps, CollectionsRelationsOps, MediaFilesReadOps},
    types::Pagination,
};
use join::JoinBuilder;
use models::{
    bulk::BulkIds,
    types::{CollectionAssetsOrdering, CollectionId},
};
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

    let mut db = ctx.db.acquire().await?;
    let ca = db
        .get_collection_assets(&id, pagination, q.ordering.unwrap_or_default())
        .await?;

    let a = db.get_assets_by_ids(&ca.ids()).await?;
    let af = db.get_assets_features_by_ids(&a.ids()).await?;
    let m = db.get_media_files_by_groups(&a.ids()).await?;
    drop(db);

    let res = JoinBuilder::new(ca)
        .with(a, |ca| ca)
        .with(af, |(_, a)| a)
        .with_group(m, |((_, a), _)| a)
        .transform(|(((ca, a), af), m)| (ca, (a, af, m)))
        .build_as(CollectionAssetDtoV1::from);
    Ok(HttpResponse::Ok().json(res))
}

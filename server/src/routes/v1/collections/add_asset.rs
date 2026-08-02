use actix_web::{HttpResponse, post, web};
use chrono::{DateTime, Utc};
use db::{
    database::DatabaseProvider,
    ops::{CollectionAssetsOps, CollectionsOps},
};
use models::{
    entities::CollectionAsset,
    types::{AssetId, CollectionAssetId, CollectionId},
};
use result::create_error;
use serde::{Deserialize, Serialize};

use crate::{di::DataCtx, routes::ApiResult};

/// Request body for adding collection asset
#[derive(Deserialize)]
struct AddCollectionAssetReq {
    asset_id: AssetId,
}

/// Response body for adding collection asset
#[derive(Serialize)]
struct AddCollectionAssetRes {
    asset_id: AssetId,
    relation: CollectionAssetId,
    added_at: DateTime<Utc>,
}

#[post("/{id}/assets")]
async fn add_collection_asset(
    id: web::Path<CollectionId>,
    payload: web::Json<AddCollectionAssetReq>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let data = payload.into_inner();

    let mut conn = ctx.db.acquire().await?;

    let collection = conn
        .get_collection(*id)
        .await?
        .ok_or(create_error!(NotFound))?;

    let rel = CollectionAsset {
        id: ctx.flake.get_id_as(),
        asset_id: data.asset_id,
        collection_id: collection.id,
        added_at: Utc::now(),
    };

    conn.insert_collection_asset(&rel).await?;
    drop(conn);

    let res = AddCollectionAssetRes {
        asset_id: rel.asset_id,
        relation: rel.id,
        added_at: rel.added_at,
    };

    Ok(HttpResponse::Created().json(res))
}

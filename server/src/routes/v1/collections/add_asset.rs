use actix_web::{HttpResponse, post, web};
use chrono::{DateTime, Utc};
use models::types::{AssetId, CollectionAssetId, CollectionId};
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

    let collection = ctx.db.collections.get_by_id(*id).await?;

    let rel = ctx
        .db
        .collections
        .add_asset(ctx.flake.get_id_as(), collection.inner.id, data.asset_id)
        .await?;

    let res = AddCollectionAssetRes {
        asset_id: rel.asset_id,
        relation: rel.id,
        added_at: rel.added_at,
    };

    Ok(HttpResponse::Created().json(res))
}

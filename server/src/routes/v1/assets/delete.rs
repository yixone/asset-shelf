use actix_web::{HttpResponse, delete, web};
use chrono::Utc;
use db::{
    database::DatabaseProvider,
    ops::{AssetsReadOps, AssetsWriteOps},
    types::patch::AssetPatch,
};
use events::AssetDeletedEvent;
use models::types::AssetId;
use result::create_error;
use serde::Serialize;

use crate::{di::DataCtx, routes::ApiResult};

/// Asset removal status
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum RemovalStateResponse {
    /// The asset was marked as deleted
    Marked,
    /// The asset was completely removed from the storage
    Deleted,
}

/// Response body for [`delete_asset`]
#[derive(Serialize)]
struct DeleteAssetResponse {
    state: RemovalStateResponse,
    id: AssetId,
}

#[delete("/{id}")]
async fn delete_asset(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let mut conn = ctx.db.acquire().await?;

    let asset = conn
        .get_asset_by_id(&id)
        .await?
        .ok_or(create_error!(NotFound))?;

    let state = match asset.deleted_at {
        Some(del) => {
            tracing::info!(id = ?id, marked_at = ?del, "Removing an already marked asset");

            conn.delete_asset(&asset.id).await?;

            ctx.events.publish(AssetDeletedEvent {
                asset: asset.id,
                media: asset.media_id,
            });

            RemovalStateResponse::Deleted
        }
        None => {
            let patch = AssetPatch::new().deleted_at(Some(Utc::now()));
            conn.update_asset(&asset.id, patch).await?;

            RemovalStateResponse::Marked
        }
    };

    let res = DeleteAssetResponse {
        state,
        id: asset.id,
    };
    Ok(HttpResponse::Ok().json(res))
}

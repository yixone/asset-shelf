use actix_web::{HttpResponse, delete, web};
use chrono::Utc;
use db::types::patch::AssetPatch;
use events::AssetDeletedEvent;
use models::types::AssetId;
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
    let asset = ctx.db.assets.get_by_id(*id).await?;

    let state = match asset.inner.deleted_at {
        Some(del) => {
            tracing::info!(id = ?id, marked_at = ?del, "Removing an already marked asset");

            ctx.db.assets.delete(*id).await?;

            ctx.events.publish(AssetDeletedEvent {
                asset: asset.inner.id,
                media: asset.inner.media_id,
            });

            RemovalStateResponse::Deleted
        }
        None => {
            let patch = AssetPatch::new().deleted_at(Some(Utc::now()));
            ctx.db.assets.update(*id, patch).await?;

            RemovalStateResponse::Marked
        }
    };

    let res = DeleteAssetResponse {
        state,
        id: asset.inner.id,
    };
    Ok(HttpResponse::Ok().json(res))
}

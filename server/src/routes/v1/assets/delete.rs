use actix_web::{HttpResponse, delete, web};
use chrono::Utc;
use db::{database::DatabaseProvider, ops::AssetOps, types::patches::AssetPatch};
use models::types::AssetId;
use result::create_error;
use serde::Serialize;
use workers::units::cleanup::CleanupWorkerTask;

use crate::{
    di::{DataCtx, EventsContext},
    routes::ApiResult,
};

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
async fn delete_asset(
    id: web::Path<AssetId>,
    ctx: web::Data<DataCtx>,
    events: web::Data<EventsContext>,
) -> ApiResult {
    let mut conn = ctx.db.acquire().await?;

    let asset = conn.get_asset(&id).await?.ok_or(create_error!(NotFound))?;

    let state = match asset.deleted_at {
        Some(del) => {
            tracing::info!(id = ?id, marked_at = ?del, "Removing an already marked asset");

            conn.delete_asset(&asset.id).await?;

            let task = CleanupWorkerTask::RemoveMedia(asset.media_id);
            events.cleanup.send(task).await;

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

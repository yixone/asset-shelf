use actix_web::{HttpResponse, patch, web};
use db::types::{UpdateResult, patch::AssetPatch};
use models::types::AssetId;
use result::create_error;
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::assets::AssetDtoV1, routes::ApiResult};

/// Request body for updating an asset
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct AssetPatchRequest {
    title: Option<String>,
    caption: Option<String>,
    source_url: Option<String>,
}

/// Updates the fields of the asset with the specified ID
///
/// ### Returns:
/// - `200` - the asset was updated
/// - `404` - asset not found
#[patch("/{id}")]
async fn patch_asset(
    id: web::Path<AssetId>,
    body: web::Json<AssetPatchRequest>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let data = body.into_inner();

    let patch = AssetPatch {
        title: data.title.into(),
        caption: data.caption.into(),
        source_url: data.source_url.into(),
        ..Default::default()
    };

    match ctx.db.assets.update(*id, patch).await? {
        // Returns the updated model
        UpdateResult::Updated(a) => {
            let res = AssetDtoV1::from(a);

            Ok(HttpResponse::Ok().json(res))
        }
        // Asset not found
        UpdateResult::NotFound => Err(create_error!(NotFound)),
    }
}

use actix_web::{HttpResponse, patch, web};
use db::{
    database::DatabaseProvider,
    ops::CollectionsOps,
    types::{UpdateResult, patches::CollectionPatch},
};
use models::types::CollectionId;
use result::create_error;
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::collections::CollectionDtoV1, routes::ApiResult};

/// Request body for updating a collection
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct CollectionPatchRequest {
    name: Option<String>,
    description: Option<String>,
}

#[patch("/{id}")]
async fn patch_collection(
    id: web::Path<CollectionId>,
    body: web::Json<CollectionPatchRequest>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let data = body.into_inner();

    // TODO: Validate empty collection name

    let patch = CollectionPatch {
        name: data.name.into(),
        description: data.description.into(),
    };

    let mut conn = ctx.db.acquire().await?;
    match conn.update_collection(*id, patch).await? {
        // Returns the updated model
        UpdateResult::Updated(c) => {
            let ca = conn
                .get_collection_additions(*id)
                .await?
                .expect("CollectionAdditions were not calculated for the existing collection");
            let res = CollectionDtoV1::from((c, ca));
            Ok(HttpResponse::Ok().json(res))
        }
        // Collection not found
        UpdateResult::NotFound => Err(create_error!(NotFound)),
    }
}

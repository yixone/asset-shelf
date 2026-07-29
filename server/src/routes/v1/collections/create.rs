use actix_web::{HttpResponse, post, web};
use chrono::Utc;
use db::{database::DatabaseProvider, ops::CollectionsOps};
use models::entities::{Collection, CollectionAdditions};
use serde::Deserialize;

use crate::{di::DataCtx, dto::v1::collections::CollectionDtoV1, routes::ApiResult};

/// Request body for creating a collections
#[derive(Deserialize)]
struct CreateCollectionRequest {
    name: String,
    description: Option<String>,
}

#[post("")]
async fn create_collection(
    body: web::Json<CreateCollectionRequest>,
    ctx: web::Data<DataCtx>,
) -> ApiResult {
    let data = body.into_inner();

    let collection = Collection {
        id: ctx.flake.get_id_as(),
        name: data.name,
        description: data.description,
        created_at: Utc::now(),
    };

    let additions = CollectionAdditions {
        collection: collection.id,
        thumbnails: Vec::new(),
        assets_count: 0,
    };

    ctx.db
        .with_session(async |db| db.insert_collection(&collection).await)
        .await?;

    let res = CollectionDtoV1::from((collection, additions));
    Ok(HttpResponse::Created().json(res))
}

use actix_web::{HttpResponse, delete, web};
use db::{database::DatabaseProvider, ops::CollectionsOps, types::DeleteResult};
use models::types::CollectionId;
use result::create_error;

use crate::{di::DataCtx, routes::ApiResult};

#[delete("/{id}")]
async fn delete_collection(id: web::Path<CollectionId>, ctx: web::Data<DataCtx>) -> ApiResult {
    // TODO: ADD SOFT DELETE

    let res = ctx
        .db
        .with_session(async |db| db.delete_collection(*id).await)
        .await?;

    match res {
        DeleteResult::Deleted(_) => Ok(HttpResponse::NoContent().finish()),
        DeleteResult::NoChanges => Err(create_error!(NotFound)),
    }
}

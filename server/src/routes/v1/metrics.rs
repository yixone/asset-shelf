use actix_web::{HttpResponse, get, web};
use result::error::ResultExt;
use telemetry::JsonEncoder;

use crate::{di::MetricsCtx, routes::ApiResult};

#[get("/metrics")]
async fn get_metrics(ctx: web::Data<MetricsCtx>) -> ApiResult {
    let collected = ctx.registry.gather();

    let encoder = JsonEncoder::new();
    let json = encoder.encode_struct(&collected).to_app_err()?;

    Ok(HttpResponse::Ok().json(json))
}

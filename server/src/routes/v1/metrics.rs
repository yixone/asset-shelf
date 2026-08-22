use actix_web::{HttpResponse, get, web};
use result::{create_error, error::ResultExt};
use telemetry::ApiTelemetryAdapter;

use crate::{
    di::{DataCtx, MetricsCtx},
    routes::ApiResult,
};

#[get("/metrics")]
async fn get_metrics(ctx: web::Data<DataCtx>, metrics: web::Data<MetricsCtx>) -> ApiResult {
    if !ctx.config.instance.allow_metrics() {
        return Err(create_error!(TelemetryDisabled));
    }

    let collected = metrics.registry.gather();

    let adapter = ApiTelemetryAdapter::new();
    let json = adapter.to_api(&collected).to_app_err()?;

    Ok(HttpResponse::Ok().json(json))
}

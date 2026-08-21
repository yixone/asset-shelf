use std::time::Instant;

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};

use crate::di::MetricsCtx;

pub async fn requests_metric_mw(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let metrics_ctx = req.app_data::<web::Data<MetricsCtx>>().cloned();

    let method = req.method().as_str().to_string();
    let route = req.match_pattern();

    let start = Instant::now();
    let res = next.call(req).await?;

    let Some(metrics) = metrics_ctx else {
        return Ok(res);
    };

    let Some(route) = route else {
        return Ok(res);
    };

    metrics
        .server
        .http_request_finished(start.elapsed(), &method, &route);

    Ok(res)
}

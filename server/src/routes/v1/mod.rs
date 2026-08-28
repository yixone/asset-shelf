use actix_web::web;

use crate::middlewares;

mod assets;
mod collections;
mod media;

mod metrics;

/// Configures endpoints for API `/v1`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .configure(assets::cfg)
            .configure(media::cfg)
            .configure(collections::cfg)
            .service(metrics::get_metrics)
            .wrap(actix_web::middleware::from_fn(
                middlewares::v1::requests_metric_mw,
            )),
    );
}

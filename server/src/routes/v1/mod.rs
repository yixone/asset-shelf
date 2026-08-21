use actix_web::web;

pub mod assets;
pub mod collections;
pub mod media;

pub mod metrics;

/// Configures endpoints for API `/v1`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .configure(assets::cfg)
            .configure(media::cfg)
            .configure(collections::cfg)
            .service(metrics::get_metrics),
    );
}

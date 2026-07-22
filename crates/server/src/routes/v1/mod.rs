use actix_web::web;

/// Configures endpoints for API `/v1`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("v1"));
}

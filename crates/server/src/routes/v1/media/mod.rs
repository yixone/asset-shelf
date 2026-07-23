use actix_web::web;

/// Configures endpoints for API `/media`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("media"));
}

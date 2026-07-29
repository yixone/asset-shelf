use actix_web::web;

mod create;

/// Configures endpoints for `/collections`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("collections").service(create::create_collection));
}

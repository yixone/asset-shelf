use actix_web::web;

pub mod get_by_id;
pub mod upload;

/// Configures endpoints for API `/assets`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("assets")
            .service(upload::upload_asset)
            .service(get_by_id::get_asset_by_id),
    );
}

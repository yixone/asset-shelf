use actix_web::web;

mod create;

mod add_asset;

/// Configures endpoints for `/collections`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("collections")
            .service(create::create_collection)
            .service(add_asset::add_collection_asset),
    );
}

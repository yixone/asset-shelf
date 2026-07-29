use actix_web::web;

mod create;

mod add_asset;
mod get_assets;
mod remove_asset;

mod get_by_id;
mod get_list;

mod delete;
mod patch;

/// Configures endpoints for `/collections`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("collections")
            .service(create::create_collection)
            .service(get_assets::get_collection_assets)
            .service(add_asset::add_collection_asset)
            .service(get_by_id::get_collection_by_id)
            .service(delete::delete_collection),
    );
}

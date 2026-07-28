use actix_web::web;

pub mod upload;

pub mod get_by_id;
pub mod get_deleted;
pub mod get_list;

pub mod patch;

pub mod delete;
pub mod restore;

pub mod similar;

/// Configures endpoints for API `/assets`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("assets")
            .service(upload::upload_asset)
            .service(patch::patch_asset)
            .service(get_deleted::get_deleted_assets)
            .service(get_list::get_assets_list)
            .service(get_by_id::get_asset_by_id)
            .service(similar::get_similar_asset)
            .service(restore::restore)
            .service(delete::delete_asset),
    );
}

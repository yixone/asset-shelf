use actix_web::web;

mod get_file;

/// Configures endpoints for API `/media`
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("media").service(get_file::get_media_file));
}

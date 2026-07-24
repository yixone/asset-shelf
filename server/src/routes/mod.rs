use actix_web::{HttpResponse, web};
use result::Result;

pub mod v1;

type ApiResult = Result<HttpResponse>;

pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.configure(v1::cfg);
}

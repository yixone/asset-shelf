use actix_web::{HttpResponse, get, web};
use result::Result;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod v1;

type ApiResult = Result<HttpResponse>;

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "Asset shelf API"), paths(ping))]
pub struct ApiDoc;

pub fn cfg(cfg: &mut web::ServiceConfig) {
    let docs = ApiDoc::openapi();

    cfg.service(ping);
    cfg.service(SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", docs));
}

#[derive(utoipa::ToSchema, serde::Serialize)]
struct PingResponse {
    message: &'static str,
}

#[utoipa::path(get, path = "/ping", responses((status = 200, body = PingResponse)))]
#[get("/ping")]
async fn ping() -> ApiResult {
    Ok(HttpResponse::Ok().json(PingResponse { message: "pong" }))
}

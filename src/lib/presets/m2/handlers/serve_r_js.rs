use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;

/// handler with path parameters like `/user/{name}/`
pub fn serve_instrumented_require_js(_req: &HttpRequest<AppState>) -> HttpResponse {
    let bytes = include_str!("../static/requirejs.js");

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(bytes)
}

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use app_state::AppState;

const INSTRUMENTED_REQUIRE_JS: &'static str = include_str!("../static/requirejs.js");

///
/// This will serve the instrumented version of RequireJS
/// that adds the meta data to each request
///
pub fn handle(_req: &HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(INSTRUMENTED_REQUIRE_JS)
}

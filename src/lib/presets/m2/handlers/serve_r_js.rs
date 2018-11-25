use actix_web::HttpRequest;
use actix_web::HttpResponse;
use app_state::AppState;
use presets::m2::opts::M2PresetOptions;
use actix_web::http::Method;
use preset::ResourceDef;

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

pub fn register(path: Option<String>) -> ResourceDef {
    let p = path.unwrap();
    (p.clone(), Method::GET, handle)
}

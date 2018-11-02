use actix_web::HttpRequest;
use actix_web::HttpResponse;
use app_state::AppState;

use presets::m2::state::gather_state;
use rjs::RequireJsBuildConfig;

///
/// This handler will serve a plain text string (should be JS?)
/// containing the code needed to create the loaders that serve
/// the additional JS
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, modules)) => {
            let module_list = RequireJsBuildConfig::bundle_loaders(
                RequireJsBuildConfig::collect_mixins(&merged_config.config),
                modules,
            );
            Ok(module_list)
        }
        Err(e) => Err(e),
    };

    match output {
        Ok(t) => HttpResponse::Ok()
            .content_type("application/javascript")
            .body(t),
        Err(e) => super::err_response::create(e),
    }
}

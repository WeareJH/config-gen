use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;

use presets::m2::preset_m2::gather_state;
use presets::m2::requirejs_config::RequireJsClientConfig;

/// serve a JSON dump of the current accumulated config
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, modules)) => {
            let module_list = RequireJsClientConfig::bundle_loaders(
                RequireJsClientConfig::mixins(&merged_config.config),
                modules,
            );
            Ok(module_list)
        }
        Err(e) => Err(e),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("text/plain").body(t),
        Err(_e) => HttpResponse::Ok().content_type("text/plain").body("NAH"),
    }
}

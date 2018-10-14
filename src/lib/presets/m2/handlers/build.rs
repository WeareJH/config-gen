use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;
use presets::m2::state::gather_state;
use serde_json;

///
/// This handler will serve up JSON that
/// can be used directly with the RequireJS optimizer.
///
/// It needs to pull together the following 3 items
///
/// 1. A log of previous request meta data
/// 2. The RequireJS Client configuration
/// 3. The bundle config (that's read from disk on request)
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, _)) => match serde_json::to_string_pretty(&merged_config) {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(e) => super::err_response::create(e),
    }
}

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use app_state::AppState;
use serde_json;

///
/// This handler will just serve back the RequireJS config as
/// it retrieved from the browser.
///
/// Note: this will be the merged-on-client version, so it
/// will need adjustments before it can be used effectively
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match req.state().rjs_client_config.lock() {
        Ok(config) => match serde_json::to_string_pretty(&*config) {
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

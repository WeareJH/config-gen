use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;
use serde_json;

///
/// serve a JSON dump of the current request log
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let modules = &req.state().req_log;
    let modules = modules.lock().unwrap();

    match serde_json::to_string_pretty(&*modules) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),
        Err(e) => super::err_response::create(e.to_string()),
    }
}

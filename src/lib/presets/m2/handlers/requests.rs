use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;

///
/// serve a JSON dump of the current request log
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let modules = &req.state().module_items;
    let modules = modules.lock().unwrap();

    let j = serde_json::to_string_pretty(&*modules).unwrap();

    HttpResponse::Ok().content_type("application/json").body(j)
}

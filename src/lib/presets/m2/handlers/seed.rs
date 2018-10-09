use actix_web::{HttpRequest, HttpResponse};
use preset::AppState;
use presets::m2::seed::SeedData;

///
/// The seed allows you to rehydrate the AppState
///
pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let req_log = &req
        .state()
        .req_log
        .lock()
        .expect("should lock & unwrap req_log");

    let client_config = req
        .state()
        .rjs_client_config
        .lock()
        .expect("should lock & unwrap rjs_client_config");

    let output = SeedData {
        rjs_client_config: client_config.clone(),
        req_log: req_log.to_vec(),
    };

    let output = match serde_json::to_string_pretty(&output) {
        Ok(t) => Ok(t),
        Err(e) => Err(e.to_string()),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(e) => HttpResponse::Ok().content_type("application/json").body(e),
    }
}

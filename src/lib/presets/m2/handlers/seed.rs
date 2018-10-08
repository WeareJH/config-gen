use actix_web::{HttpRequest, HttpResponse};
use preset::AppState;
use presets::m2::seed::SeedData;

pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let module_items = &req
        .state()
        .module_items
        .lock()
        .expect("should lock & unwrap module_items");

    let client_config = req
        .state()
        .require_client_config
        .lock()
        .expect("should lock & unwrap require_client_config");

    let output = SeedData {
        client_config: client_config.clone(),
        module_items: module_items.to_vec(),
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

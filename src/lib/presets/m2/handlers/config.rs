use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;

pub fn handle(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match req.state().require_client_config.lock() {
        Ok(config) => match serde_json::to_string_pretty(&*config) {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(_e) => HttpResponse::Ok()
            .content_type("application/json")
            .body("Could not serve config"),
    }
}

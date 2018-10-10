use actix_web::http::StatusCode;
use actix_web::HttpResponse;

pub fn create(message: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .status(StatusCode::from_u16(500).expect("can set 500 resp code"))
        .body(
            serde_json::to_string_pretty(&json!({
                "message": message
            })).unwrap(),
        )
}
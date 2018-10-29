use actix_web::{AsyncResponder, HttpMessage};
use actix_web::{HttpRequest, HttpResponse};
use app_state::AppState;
use futures::{Future, Stream};
use presets::m2::preset_m2::FutResp;
use rjs::RequireJsClientConfig;
use std;

///
/// This handler accepts the incoming RequireJS merged
/// config from the client
///
pub fn handle(original_request: &HttpRequest<AppState>) -> FutResp {
    let client_config_clone = original_request.state().rjs_client_config.clone();

    original_request
        .payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let c2 = client_config_clone.clone();
            match std::str::from_utf8(&body[..]) {
                Ok(body) => match RequireJsClientConfig::update_in_place(body.to_string(), c2) {
                    Ok(()) => Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body("ok")),
                    Err(e) => Ok(super::err_response::create(e.to_string())),
                },
                Err(e) => Ok(super::err_response::create(e.to_string())),
            }
        }).responder()
}

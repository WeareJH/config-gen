use actix_web::{AsyncResponder, HttpMessage};
use actix_web::{HttpRequest, HttpResponse};
use app_state::AppState;
use futures::{Future, Stream};
use presets::m2::preset_m2::FutResp;
use presets::m2::requirejs_config::RequireJsClientConfig;
use serde_json;
use std;

///
/// This handler accepts the incoming RequireJS merged
/// config from the client
///
pub fn handle(req: &HttpRequest<AppState>) -> FutResp {
    let a = req.state().rjs_client_config.clone();

    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let result: Result<RequireJsClientConfig, serde_json::Error> =
                serde_json::from_str(std::str::from_utf8(&body).unwrap());
            //
            match result {
                Ok(next_config) => {
                    let mut mutex = a.lock().unwrap();
                    mutex.base_url = next_config.base_url;
                    mutex.map = next_config.map;
                    mutex.config = next_config.config;
                    mutex.paths = next_config.paths;
                    mutex.shim = next_config.shim;

                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body("ok"))
                }
                Err(e) => Ok(super::err_response::create(e.to_string())),
            }
        }).responder()
}

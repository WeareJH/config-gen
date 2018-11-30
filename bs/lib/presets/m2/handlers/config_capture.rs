use actix_web::http::Method;
use actix_web::HttpRequest;
use app_state::AppState;
use preset::AsyncResourceDef;
use presets::m2::preset_m2::FutResp;
use proxy_utils::apply_to_proxy_body;
use rjs::RequireJsClientConfig;

///
/// This handler will record the incoming string from the Magento-generated
/// requirejs-config.js and use that to build up the 'deps' array. This is required
/// since the client config that the client posts back does not include all original
/// 'deps' (I'm not sure why)
///
pub fn handle(original_request: &HttpRequest<AppState>) -> FutResp {
    let client_config_clone = original_request.state().rjs_client_config.clone();
    apply_to_proxy_body(&original_request, move |b| {
        let c2 = client_config_clone.clone();
        match RequireJsClientConfig::update_in_place(b.to_string(), c2) {
            Ok(..) => { /* no op */ }
            Err(e) => {
                eprintln!(
                    "Could not update `RequireJsClientConfig` in place, e = {}",
                    e
                );
            }
        };
        b
    })
}

pub fn register(path: Option<String>) -> AsyncResourceDef {
    let p = path.unwrap();
    (p.clone(), Method::GET, handle)
}

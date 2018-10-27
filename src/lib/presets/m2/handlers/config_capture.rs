use actix_web::HttpRequest;
use app_state::AppState;
use presets::m2::preset_m2::FutResp;
use presets::m2::requirejs_config::RequireJsClientConfig;
use proxy_utils::apply_to_proxy_body;

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
        if let Ok(rjs) = RequireJsClientConfig::from_generated_string(b.to_string()) {
            let mut w = c2.lock().expect("unwrapped client_config_clone");
            *w = rjs
        }
        b
    })
}

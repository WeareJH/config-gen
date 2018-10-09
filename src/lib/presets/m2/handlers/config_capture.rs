use actix_web::HttpRequest;
use preset::AppState;
use presets::m2::parse::get_deps_from_str;
use presets::m2::preset_m2::FutResp;
use proxy_utils::apply_to_proxy_body;

///
/// This handler has 2 purposes.
///
/// First, it will record the incoming string from the Magento-generated
/// requirejs-config.js and use that to build up the 'deps' array. This is required
/// since the client config that the client posts back does not include all original
/// 'deps' (I'm not sure why)
///
/// Secondly, it will append a small piece of JS to the end of the file in order
/// to send back the configuration.
///
pub fn handle(original_request: &HttpRequest<AppState>) -> FutResp {
    let client_config_clone = original_request.state().rjs_client_config.clone();
    apply_to_proxy_body(&original_request, move |mut b| {
        let c2 = client_config_clone.clone();
        if let Ok(deps) = get_deps_from_str(&b) {
            let mut w = c2.lock().expect("unwraped");
            w.deps = deps;
        };
        b.push_str(include_str!("../static/post_config.js"));
        b
    })
}

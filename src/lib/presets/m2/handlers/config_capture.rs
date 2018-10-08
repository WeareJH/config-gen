use actix_web::HttpRequest;
use preset::AppState;
use presets::m2::parse::get_deps_from_str;
use presets::m2::preset_m2::{apply_to_proxy_body, FutResp};

pub fn handle(original_request: &HttpRequest<AppState>) -> FutResp {
    let client_config_clone = original_request.state().require_client_config.clone();
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

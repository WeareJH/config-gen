use actix_web::client::ClientResponse;
use actix_web::{AsyncResponder, Error, HttpMessage, HttpRequest};
use app_state::AppState;
use futures::Future;
use presets::m2::preset_m2::FutResp;
use proxy_transform::create_outgoing;
use proxy_transform::get_host_port;
use proxy_transform::proxy_req_setup;

///
/// A helper for applying a transformation on a proxy
/// response before sending it back to the origin requester
///
pub fn apply_to_proxy_body<F>(original_request: &HttpRequest<AppState>, f: F) -> FutResp
where
    F: Fn(String) -> String + 'static,
{
    let mut outgoing = proxy_req_setup(original_request);
    let target_domain = original_request.state().opts.target.clone();
    let bind_port = original_request.state().opts.port;
    let (host, port) = get_host_port(original_request, bind_port);

    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |proxy_response: ClientResponse| {
            proxy_response
                .body()
                .limit(1_000_000)
                .from_err()
                .and_then(move |body| {
                    use std::str;

                    let req_target = format!("{}:{}", host, port);
                    let body_content = str::from_utf8(&body[..]).unwrap();
                    let next_body: String = String::from(body_content);

                    Ok(create_outgoing(
                        &proxy_response.status(),
                        &proxy_response.headers(),
                        target_domain.to_string(),
                        req_target,
                    )
                    .body(f(next_body)))
                })
        })
        .responder()
}

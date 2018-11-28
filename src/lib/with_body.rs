use actix_web::client::ClientRequestBuilder;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use app_state::AppState;
use futures::Future;
use proxy_transform::create_outgoing;
use std::time::Duration;

///
/// This case handles incoming POST requests
/// that contain a body.
///
/// Note: This is not tested in any way with large uploads
///
pub fn forward_request_with_body(
    incoming_request: &HttpRequest<AppState>,
    req_target: String,
    mut outgoing: ClientRequestBuilder,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = incoming_request.state();
    let timeout: u64 = state.opts.proxy_timeout_secs.into();
    let next_target = incoming_request.state().opts.target.clone();
    let output = incoming_request
        .body()
        .from_err()
        .and_then(move |incoming_body| {
            outgoing
                .body(incoming_body)
                .unwrap()
                .send()
                .timeout(Duration::from_secs(timeout))
                .map_err(Error::from)
                .and_then(move |proxy_response| {
                    proxy_response
                        .body()
                        .from_err()
                        .and_then(move |proxy_response_body| {
                            Ok(create_outgoing(
                                &proxy_response.status(),
                                &proxy_response.headers(),
                                next_target.to_string(),
                                req_target,
                            )
                            .body(proxy_response_body))
                        })
                })
        });

    Box::new(output)
}

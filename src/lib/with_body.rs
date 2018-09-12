use futures::{Future};
use actix_web::{
    Error, HttpMessage,
    HttpRequest, HttpResponse
};
use options::ProxyOpts;
use actix_web::client::ClientRequestBuilder;
use fns::create_outgoing;

///
/// This case handles incoming POST requests
/// that contain a body.
///
/// Note: This is not tested in any way with large uploads
///
pub fn forward_request_with_body(_req: &HttpRequest<ProxyOpts>, mut outgoing: ClientRequestBuilder) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let next_target = _req.state().target.clone();
    let next_host = _req.uri().clone();
    let output = _req.body()
        .from_err()
        .and_then(move |incoming_body| {
            outgoing.body(incoming_body).unwrap().send().map_err(Error::from)
                .and_then(move |proxy_response| {
                    let req_host = next_host.host().unwrap_or("");
                    let req_port = next_host.port().unwrap_or(80);
                    let req_target = format!("{}:{}", req_host, req_port);
                    proxy_response.body()
                        .from_err()
                        .and_then(move |proxy_response_body| {
                            Ok(create_outgoing(
                                &proxy_response.headers(),
                                next_target.to_string(),
                                req_target
                            ).body(proxy_response_body))
                        })
                })
        });

    Box::new(output)
}

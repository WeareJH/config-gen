use futures::{Future, Stream};
use futures::future::{Either, ok};
use actix_web::{
    client, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev
};
use options::ProxyOpts;
use actix_web::http::header;
use actix_web::client::ClientRequestBuilder;
use fns::create_outgoing;
use rewrites::{replace_cookie_domain_on_page, RewriteContext};
use rewrites::replace_host;

pub fn forward_request_without_body(_req: &HttpRequest<ProxyOpts>, mut outgoing: ClientRequestBuilder) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let next_target = _req.state().target.clone();
    let next_host = _req.uri().clone();
    outgoing.finish().unwrap().send().map_err(Error::from)
        .and_then(move |proxy_response| {

            // Should we rewrite this response?
            // just check for the correct content-type header for now.
            // This will need fleshing out to provide stricter checks
            let rewrite_response = match proxy_response.headers().get(header::CONTENT_TYPE) {
                Some(t) => {
                    match t.to_str().unwrap_or("") {
                        "text/html" | "text/html; charset=UTF-8" => true,
                        _ => false,
                    }
                }
                _ => false
            };

            // If we decide to modify the response, we need to buffer the entire
            // response into memory (text files only)
            if rewrite_response {
                Either::A(
                    proxy_response.body()
                        .from_err()
                        .and_then(move |body| {
                            use std::str;

                            // In here, we now have a ful buffered response body
                            // so we can go ahead and apply URL replacements
                            let req_host = next_host.host().unwrap_or("");
                            let req_port = next_host.port().unwrap_or(80);
                            let req_target = format!("{}:{}", req_host, req_host);
                            let context = RewriteContext {
                                host_to_replace: &next_target,
                                target_host: req_host,
                                target_port: req_port
                            };
                            let next_body = replace_host(
                                str::from_utf8(&body[..]).unwrap(),
                                &context
                            );
                            let next_body = replace_cookie_domain_on_page(&next_body, &context);
                            let as_string = next_body.to_string();
                            Ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(as_string))
                        })
                )
            } else {
                let req_host = next_host.host().unwrap_or("");
                let req_port = next_host.port().unwrap_or(80);
                let req_target = format!("{}:{}", req_host, req_host);
                // If we get here, we decided not to re-write the response
                // so we just stream it back to the client
                Either::B(
                    ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(Body::Streaming(Box::new(proxy_response.payload().from_err()))))
                )
            }
        })
        .responder()
}
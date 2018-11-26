use actix_web::client::ClientRequestBuilder;
use actix_web::client::ClientResponse;
use actix_web::http::header;
use actix_web::http::uri::Uri;
use actix_web::{AsyncResponder, Body, Error, HttpMessage, HttpRequest, HttpResponse};
use futures::future::{ok, Either};
use futures::{Future, Stream};

use std::time::Duration;
use app_state::AppState;
use preset::RewriteFns;
use proxy_transform::create_outgoing;
use proxy_transform::get_host_port;
use replacer::{Replacer, Subject};
use rewrites::{replace_host, RewriteContext};

///
/// Process regular GET requests where we don't need to consider
/// the request BODY
///
pub fn forward_request_without_body(
    incoming_request: &HttpRequest<AppState>,
    req_target: String,
    mut outgoing: ClientRequestBuilder,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let target_domain = incoming_request.state().opts.target.clone();
    let bind_port = incoming_request.state().opts.port;
    let req_uri = incoming_request.uri().clone();
    let rewrites = incoming_request.state().rewrites.clone();

    let (host, port) = get_host_port(incoming_request, bind_port);

    outgoing
        .finish()
        .unwrap()
        .send()
        .timeout(Duration::from_secs(5))
        .map_err(Error::from)
        .and_then(move |proxy_response: ClientResponse| {
            debug!("Got proxy response, status={}", proxy_response.status());
            debug!(
                "Got proxy response headers, headers={:#?}",
                proxy_response.headers()
            );

            // If we decide to modify the response, we need to buffer the entire
            // response into memory (text content only)
            if should_rewrite_body(&req_uri, &proxy_response) {
                debug!("attempting to rewrite body");
                Either::A(response_from_rewrite(
                    proxy_response,
                    host,
                    port,
                    req_target,
                    target_domain,
                    rewrites,
                ))
            } else {
                // If we get here, we decided not to re-write the response
                // so we just stream it back to the client
                Either::B(pass_through_response(
                    proxy_response,
                    req_target.clone(),
                    target_domain,
                ))
            }
        }).responder()
}

/// Pass-through response
fn pass_through_response(
    proxy_response: ClientResponse,
    req_target: String,
    target_domain: String,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let output = ok(create_outgoing(
        &proxy_response.status(),
        &proxy_response.headers(),
        target_domain.to_string(),
        req_target,
    ).body(Body::Streaming(Box::new(
        proxy_response.payload().from_err(),
    ))));

    Box::new(output)
}

///
/// Create a response, after buffering & rewriting the
/// response received from the proxy target
///
fn response_from_rewrite(
    proxy_response: ClientResponse,
    req_host: String,
    req_port: u16,
    req_target: String,
    target_domain: String,
    rewrites: RewriteFns,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let output = proxy_response
        .body()
        .limit(1_000_000)
        .from_err()
        .and_then(move |body| {
            use std::str;

            let context = RewriteContext {
                host_to_replace: target_domain.clone(),
                target_host: req_host,
                target_port: req_port,
            };

            let body_content = str::from_utf8(&body[..]).unwrap();

            // Append any rewrites from presets
            let mut fns: RewriteFns = vec![replace_host];
            fns.extend(rewrites);
            let next_body = Subject::new(body_content).apply(&context, fns);

            debug!("creating response");

            Ok(create_outgoing(
                &proxy_response.status(),
                &proxy_response.headers(),
                target_domain.to_string(),
                req_target,
            ).body(next_body))
        });

    Box::new(output)
}

///
/// Determine if the current request should be rewritten
/// Currently this just checks for a header of type text/html
///
fn should_rewrite_body(uri: &Uri, resp: &ClientResponse) -> bool {
    if uri.path().contains("requirejs-config.js") {
        return true;
    }

    resp.headers()
        .get(header::CONTENT_TYPE)
        .map_or(false, |header_value| {
            match header_value.to_str().and_then(|s| Ok(s.to_lowercase())) {
                Ok(s) => match s.as_str() {
                    "text/html" | "text/html; charset=utf-8" => true,
                    _ => false,
                },
                Err(..) => false,
            }
        })
}

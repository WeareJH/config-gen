use actix_web::client::ClientRequestBuilder;
use actix_web::client::ClientResponse;
use actix_web::http::header;
use actix_web::http::uri::Uri;
use actix_web::http::HeaderMap;
use actix_web::{AsyncResponder, Body, Error, HttpMessage, HttpRequest, HttpResponse};
use futures::future::{ok, Either};
use futures::{Future, Stream};

use fns::create_outgoing;
use options::ProxyOpts;
use replacer::{Replacer, Subject};
use rewrites::{replace_cookie_domain_on_page, replace_host, RewriteContext};

///
/// Process regular GET requests where we don't need to consider
/// the request BODY
///
pub fn forward_request_without_body(
    incoming_request: &HttpRequest<ProxyOpts>,
    mut outgoing: ClientRequestBuilder,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let target_domain = incoming_request.state().target.clone();
    let req_uri = incoming_request.uri().clone();

    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |proxy_response| {
            // If we decide to modify the response, we need to buffer the entire
            // response into memory (text content only)
            if should_rewrite_body(proxy_response.headers()) {
                Either::A(response_from_rewrite(
                    proxy_response,
                    req_uri,
                    target_domain,
                ))
            } else {
                // If we get here, we decided not to re-write the response
                // so we just stream it back to the client
                Either::B(pass_through_response(
                    proxy_response,
                    req_uri,
                    target_domain,
                ))
            }
        }).responder()
}

/// Pass-through response
fn pass_through_response(
    proxy_response: ClientResponse,
    req_uri: Uri,
    target_domain: String,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let req_host = req_uri.host().unwrap_or("");
    let req_port = req_uri.port().unwrap_or(80);
    let req_target = format!("{}:{}", req_host, req_port);

    let output = ok(create_outgoing(
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
    req_uri: Uri,
    target_domain: String,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let next_host = req_uri.clone();

    let output = proxy_response.body().from_err().and_then(move |body| {
        use std::str;

        // In here, we now have a ful buffered response body
        // so we can go ahead and apply URL replacements
        let req_host = next_host.host().unwrap_or("");
        let req_port = next_host.port().unwrap_or(80);
        let req_target = format!("{}:{}", req_host, req_host);
        let context = RewriteContext {
            host_to_replace: target_domain.clone(),
            target_host: String::from(req_host),
            target_port: req_port,
        };
        let body_content = str::from_utf8(&body[..]).unwrap();
        let subject = Subject::new(body_content)
            .apply(&context, vec![replace_host, replace_cookie_domain_on_page]);
        Ok(create_outgoing(
            &proxy_response.headers(),
            target_domain.to_string(),
            req_target,
        ).body(subject))
    });

    Box::new(output)
}

///
/// Determine if the current request should be rewritten
/// Currently this just checks for a header of type text/html
///
fn should_rewrite_body(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .map_or(false, |header_value| {
            match header_value.to_str().unwrap_or("") {
                "text/html" | "text/html; charset=UTF-8" => true,
                _ => false,
            }
        })
}

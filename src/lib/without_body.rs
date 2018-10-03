use actix_web::client::ClientRequestBuilder;
use actix_web::client::ClientResponse;
use actix_web::http::header;
use actix_web::http::uri::Uri;
use actix_web::{AsyncResponder, Body, Error, HttpMessage, HttpRequest, HttpResponse};
use futures::future::{ok, Either};
use futures::{Future, Stream};

use preset::AppState;
use preset::RewriteFns;
use proxy_transform::create_outgoing;
use replacer::{Replacer, Subject};
use rewrites::{replace_host, RewriteContext};

///
/// Process regular GET requests where we don't need to consider
/// the request BODY
///
pub fn forward_request_without_body(
    incoming_request: &HttpRequest<AppState>,
    mut outgoing: ClientRequestBuilder,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let target_domain = incoming_request.state().opts.target.clone();
    let bind_port = incoming_request.state().opts.port;
    let req_uri = incoming_request.uri().clone();
    let rewrites = incoming_request.state().rewrites.clone();

    let split = match incoming_request.headers().get(header::HOST) {
        Some(h) => {
            let output: Vec<&str> = h.to_str().expect("host to str").split(":").collect();
            output
        }
        None => vec![],
    };

    let (host, port) = match (split.get(0), split.get(1)) {
        (Some(h), Some(p)) => (h.to_string(), p.parse().expect("parsed port")),
        (Some(h), None) => (h.to_string(), 80 as u16),
        _ => ("127.0.0.1".to_string(), bind_port),
    };

    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |proxy_response: ClientResponse| {
            // If we decide to modify the response, we need to buffer the entire
            // response into memory (text content only)
            if should_rewrite_body(&req_uri, &proxy_response) {
                Either::A(response_from_rewrite(
                    proxy_response,
                    req_uri,
                    host,
                    port,
                    target_domain,
                    rewrites,
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
        })
        .responder()
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
    req_host: String,
    req_port: u16,
    target_domain: String,
    rewrites: RewriteFns,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let next_host = req_uri.clone();
    println!("{:?}", next_host);

    let output = proxy_response
        .body()
        .limit(1_000_000)
        .from_err()
        .and_then(move |body| {
            use std::str;

            let is_require_config = next_host.path().contains("requirejs-config.js");
            let req_target = format!("{}:{}", req_host, req_port);
            let context = RewriteContext {
                host_to_replace: target_domain.clone(),
                target_host: req_host,
                target_port: req_port,
            };

            let body_content = str::from_utf8(&body[..]).unwrap();

            let next_body: String = if is_require_config {
                let mut b = String::from(body_content);
                b.push_str(
                    r#"
var xhr = new XMLHttpRequest();

xhr.open('POST', '/__bs/post');
xhr.setRequestHeader('Content-Type', 'application/json');
xhr.onload = function() {
    if (xhr.status === 200) {
        console.log('sent');
    }
    else if (xhr.status !== 200) {
        alert('Request failed.  Returned status of ' + xhr.status);
    }
};
xhr.send(JSON.stringify(requirejs.s.contexts._.config));
                "#,
                );
                b
            } else {
                // Append any rewrites from presets
                let mut fns: RewriteFns = vec![replace_host];
                fns.extend(rewrites);
                Subject::new(body_content).apply(&context, fns)
            };

            Ok(create_outgoing(
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
            match header_value.to_str().unwrap_or("") {
                "text/html" | "text/html; charset=UTF-8" => true,
                _ => false,
            }
        })
}

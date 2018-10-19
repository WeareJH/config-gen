use actix::Actor;
use actix_web::client::ClientConnector;
use actix_web::client::ClientRequestBuilder;
use actix_web::http::StatusCode;
use actix_web::http::{header, HeaderMap, Method};
use actix_web::{client, dev, http, Error, HttpMessage, HttpRequest, HttpResponse};
use app_state::AppState;
use base64::encode;
use futures::Future;
use headers::clone_headers;
use openssl::ssl::SslConnector;
use openssl::ssl::{SslMethod, SslVerifyMode};
use presets::m2::opts::{AuthBasic, M2PresetOptions};
use std::str;
use with_body::forward_request_with_body;
use without_body::forward_request_without_body;

///
/// This function will clone incoming requests
/// and pass them onto a backend specified via the `target` field on [ProxyOpts]
///
pub fn proxy_transform(
    original_request: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let outgoing = proxy_req_setup(original_request);
    let bind_port = original_request.state().opts.port;
    let scheme = &original_request.state().opts.scheme;
    let (host, port) = get_host_port(original_request, bind_port);
    let req_target = format!("{}://{}:{}", scheme, host, port);

    match *original_request.method() {
        Method::POST => forward_request_with_body(original_request, req_target, outgoing),
        _ => forward_request_without_body(original_request, req_target, outgoing),
    }
}

pub fn proxy_req_setup(original_request: &HttpRequest<AppState>) -> ClientRequestBuilder {
    debug!(
        "incoming proxy_req = {:?}",
        original_request.connection_info().host()
    );
    let original_req_headers = original_request.headers().clone();
    let cloned = clone_headers(
        &original_req_headers,
        original_request.connection_info().host().to_string(),
        original_request.state().opts.target.clone(),
    );

    // build up the next outgoing URL (for the back-end)
    let next_url = format!(
        "{}://{}{}{}",
        original_request.state().opts.scheme,
        original_request.state().opts.target,
        original_request.path(),
        match original_request.uri().query().as_ref() {
            Some(q) => format!("?{}", q),
            None => "".to_string(),
        }
    );

    debug!("next_url={}", next_url);

    let mut outgoing = client::ClientRequest::build();

    // Since this is a development tool only, we're being risky here
    // and just disabling all SSL verifications
    let mut ssl_conn = SslConnector::builder(SslMethod::tls()).unwrap();
    ssl_conn.set_verify(SslVerifyMode::NONE);

    let conn = ClientConnector::with_connector(ssl_conn.build()).start();

    outgoing
        .with_connector(conn)
        .method(original_request.method().clone())
        .uri(next_url);

    for (key, value) in cloned.iter() {
        outgoing.header(key.clone(), value.clone());
    }

    // ensure the 'host' header is re-written
    outgoing.set_header(
        http::header::HOST,
        original_request.state().opts.target.clone(),
    );

    outgoing.set_header(
        http::header::ORIGIN,
        original_request.state().opts.target.clone(),
    );

    // combine all cookie headers into a single one
    let joined_cookie = original_req_headers
        .get_all(header::COOKIE)
        .iter()
        .map(|hdr| {
            let s = str::from_utf8(hdr.as_bytes()).unwrap_or("");
            s.to_string()
        }).collect::<Vec<String>>()
        .join("; ");

    outgoing.set_header(http::header::COOKIE, joined_cookie);

    //
    // Add basic auth header if auth_basic is present in the options
    //
    M2PresetOptions::get_opts(&original_request.state().program_config).map(|opts| {
        opts.auth_basic.map(|auth: AuthBasic| {
            let combined = format!("{}:{}", auth.username, auth.password);
            outgoing.set_header(
                http::header::AUTHORIZATION,
                format!("Basic {}", encode(&combined)),
            );
        });
    });

    outgoing
}

pub fn create_outgoing(
    status_code: &StatusCode,
    resp_headers: &HeaderMap,
    target: String,
    replacer: String,
) -> dev::HttpResponseBuilder {
    let mut outgoing = HttpResponse::Ok();
    outgoing.status(*status_code);
    let c = clone_headers(resp_headers, target, replacer);
    debug!("Headers for response = {:#?}", c);
    // Copy headers from backend response to main response
    for (key, value) in c.iter() {
        outgoing.header(key.clone(), value.clone());
    }
    outgoing
}

pub fn get_host_port(incoming_request: &HttpRequest<AppState>, bind_port: u16) -> (String, u16) {
    let info = incoming_request.connection_info();
    let split: Vec<&str> = info.host().split(":").collect();

    match (split.get(0), split.get(1)) {
        (Some(h), Some(p)) => (h.to_string(), p.parse().expect("parsed port")),
        (Some(h), None) => (h.to_string(), bind_port),
        _ => ("127.0.0.1".to_string(), bind_port),
    }
}

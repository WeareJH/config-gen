#![allow(unused_imports)]
#![allow(dead_code)]
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate bytes;

use actix_web::{
    client, middleware, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev
};
use futures::{Future, Stream};
use std::path::PathBuf;
use std::sync::Arc;
use std::net::SocketAddr;

///
/// # Examples
///
/// ```
/// # use bs_rust::*;
/// let opts = ProxyOpts::new("example.com");
/// assert_eq!(opts.target, "example.com".to_string());
/// ```
///
#[derive(Clone)]
pub struct ProxyOpts {
    pub target: String
}

impl ProxyOpts {
    pub fn new(target: impl Into<String>) -> ProxyOpts {
        ProxyOpts { target: target.into() }
    }
}

///
/// This function will clone incoming requests
/// and pass them onto a backend specified via the `target` field on [ProxyOpts]
///
pub fn proxy_transform(_req: &HttpRequest, opts: ProxyOpts) -> Box<Future<Item = HttpResponse, Error = Error>> {

    // this is a placeholder for some logic to determine if we need to
    // modify the response body.
    let rewrite_response = false;

    // building up the new request that we'll send to the backend
    let mut outgoing = client::ClientRequest::build_from(_req);

    // reset the uri so that it points to the correct proxied host + path
    outgoing.uri(format!("http://{}{}", opts.target.clone(), _req.uri()).as_str());

    // ensure the 'host' header is re-written
    outgoing.set_header(http::header::HOST, opts.target.clone());

    // The shared parts of the response builder
    let setup = outgoing.finish().unwrap().send().map_err(Error::from);

    // now choose how to handle it
    if rewrite_response {
        // if the client responds with a request we want to alter (such as HTML)
        // then we need to buffer the body into memory in order to apply regex's on the string
        setup.and_then(|resp| {
            resp.body()
                .from_err()
                .and_then(move |body| {
                    // now we're not rewriting anything, but we could since
                    // here the 'body' is the entire response body
                    Ok(create_outgoing(&resp).body(body))
                })
        }).responder()
    } else {
        // The streaming response is simpler, we just need to copy the headers
        // over and then stream the result back
        setup.and_then(|resp| {
            Ok(create_outgoing(&resp).body(Body::Streaming(Box::new(resp.payload().from_err()))))
        }).responder()
    }
}

fn create_outgoing(client_response: &client::ClientResponse) -> dev::HttpResponseBuilder {
    let mut outgoing = HttpResponse::Ok();
    // Copy headers from backend response to main response
    for (key, value) in client_response.headers() {
        outgoing.header(key.clone(), value.clone());
    }
    outgoing
}
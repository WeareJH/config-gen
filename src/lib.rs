#![allow(unused_imports)]
#![allow(dead_code)]
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate bytes;

use actix_web::{
    client, middleware, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http
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
    #[derive(Copy)]
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

    // now finish the request builder and execute it
    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |resp| {

            let mut outgoing = HttpResponse::Ok();

            // Copy headers from backend response to main response
            for (key, value) in resp.headers() {
                outgoing.header(key.clone(), value.clone());
            }

            // here, if I want to rewrite the response, I need to
            // buffer the response body and then send it back
            if rewrite_response {
                resp.body()
                    .from_err()
                    .and_then(move |body| {
                        Ok(HttpResponse::Ok().body(body))
                    })
            } else {
                // otherwise, this will just stream all responses from the proxy
                // back to the browser without touching them
                Ok(outgoing.body(Body::Streaming(Box::new(resp.payload().from_err()))))
            }
        })
        .responder()
}

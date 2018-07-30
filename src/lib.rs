#![allow(unused_imports)]
#![allow(dead_code)]
extern crate actix_web;
extern crate futures;

use actix_web::{HttpRequest, HttpResponse, Error, client, test, HttpMessage, AsyncResponder};
use futures::Future;
use std::sync::Arc;
use std::net::SocketAddr;
use actix_web::http::header;

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

    println!("proxy handler: {:#?}", _req.headers());

    // create a new request from the incoming
    // this will copy method/header/cookies automatically
    let mut outgoing = client::ClientRequest::build_from(_req);

    // reset the uri so that it points to the correct proxied host
    outgoing.uri(format!("http://{}", opts.target.clone()).as_str());

    // ensure the 'host' header is re-written
    outgoing.set_header(header::HOST, opts.target.clone());

    // now finish the request builder and execute it
    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            println!("proxy RESP: {:#?}", resp);
            resp.body()
                .from_err()
                .and_then(move |body| {
                    let mut outgoing = HttpResponse::Ok();

                    // Copy headers from backend response to main response
                    for (key, value) in resp.headers() {
                        outgoing.header(key.clone(), value.clone());
                    }

                    Ok(outgoing.body(body))
                })
        })
        .responder()
}

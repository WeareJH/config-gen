use actix_web::{
    client, middleware, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev
};
use futures::{Future, Stream};
use std::path::PathBuf;
use std::sync::Arc;
use std::net::SocketAddr;
use rewrites::replace_host;

///
/// # Examples
///
/// ```
/// # use bs_rust::*;
/// let opts = ProxyOpts::new("example.com", "127.0.0.:8000");
/// assert_eq!(opts.target, "example.com".to_string());
/// ```
///
#[derive(Clone)]
pub struct ProxyOpts {
    pub target: String,
    pub host: String,
}

impl ProxyOpts {
    pub fn new(target: impl Into<String>, host: impl Into<String>) -> ProxyOpts {
        ProxyOpts { target: target.into(), host: host.into() }
    }
}

///
/// This function will clone incoming requests
/// and pass them onto a backend specified via the `target` field on [ProxyOpts]
///
pub fn proxy_transform(_req: &HttpRequest, opts: ProxyOpts) -> Box<Future<Item = HttpResponse, Error = Error>> {

    // this is a placeholder for some logic to determine if we need to
    // modify the response body.
    let rewrite_response = _req.path() == "/";

    // building up the new request that we'll send to the backend
    let mut outgoing = client::ClientRequest::build_from(_req);

    let next_url = format!("{}://{}{}{}",
                      match _req.uri().scheme_part() {
                          Some(scheme) => scheme.as_str(),
                          None => "http"
                      },
                      opts.target.clone(),
                      _req.path(),
                      match _req.uri().query().as_ref() {
                          Some(q) => format!("?{}", q),
                          None => "".to_string()
                      });

    // reset the uri so that it points to the correct proxied host + path
    outgoing.uri(next_url.as_str());

    // ensure the 'host' header is re-written
    outgoing.set_header(http::header::HOST, opts.target.clone());

    // The shared parts of the response builder
    let setup = outgoing.finish().unwrap().send().map_err(Error::from);

    // now choose how to handle it
    if rewrite_response {
        // if the client responds with a request we want to alter (such as HTML)
        // then we need to buffer the body into memory in order to apply regex's on the string
        let next_target = opts.target.clone();
        let next_host = opts.host.clone();
        setup.and_then(|resp| {
            resp.body()
                .limit(1_000_000)
                .from_err()
                .and_then(move |body| {
                    // now we're not rewriting anything, but we could since
                    // here the 'body' is the entire response body
                    use std::str;

                    let next_body = replace_host(str::from_utf8(&body[..]).unwrap(), &next_host, &next_target);
                    let as_string = next_body.to_string();
                    Ok(create_outgoing(&resp).body(as_string))
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
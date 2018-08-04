use actix_web::{
    client, middleware, server, App, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev, test
};
use actix_web::http::header;
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
/// let opts = ProxyOpts::new("example.com", "127.0.0.1:8000");
/// assert_eq!(opts.target, "example.com".to_string());
/// ```
///
#[derive(Clone)]
pub struct ProxyOpts {
    pub target: String,
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
        let next_host = _req.uri().clone();
        setup.and_then(move |resp| {
            resp.body()
                .limit(1_000_000)
                .from_err()
                .and_then(move |body| {
                    // now we're not rewriting anything, but we could since
                    // here the 'body' is the entire response body
                    use std::str;

                    let req_host = next_host.host().unwrap_or("");
                    let req_port = next_host.port().unwrap_or(80);
                    let next_body = replace_host(
                        str::from_utf8(&body[..]).unwrap(),
                        &next_target,
                        req_host, req_port
                    );
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

const STR: &str = "Hello world";
const STR2: &str = "Hello world 2";

#[test]
fn test_forwards_headers() {
    let server = test::TestServer::new(|app| {
        app.handler(|req: &HttpRequest| {
            println!("headers received at proxy addr: {:#?}", req.headers());
            assert_eq!(req.headers().get(header::ACCEPT).unwrap(), "text/html");
            HttpResponse::Ok()
                .header("shane", "kittens")
                .body(STR)
        })
    });

    let srv_address = server.addr().to_string();
    println!("orig address = {}", srv_address);

    let mut proxy = test::TestServer::new(move |app| {
        let addr = srv_address.clone();
        app.handler(move |req| proxy_transform(req, ProxyOpts::new(addr.clone())));
    });

    let request = proxy.get()
        .header(header::ACCEPT, "text/html")
        .uri(proxy.url("/"))
        .finish()
        .unwrap();

    let response = proxy.execute(request.send()).unwrap();
    let _bytes = proxy.execute(response.body()).unwrap();

    println!("main resp: {:#?}", response.headers());

    let has_header = response.headers().get("shane").unwrap();

    assert_eq!(has_header, "kittens");
}

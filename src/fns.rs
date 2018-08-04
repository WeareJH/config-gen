use actix_web::{
    client, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev, test,
};
use actix_web::http::header;
use futures::{Future, Stream};
use futures::future::{Either, ok};
use rewrites::replace_host;
use mime::TEXT_HTML;

///
/// # Examples
///
/// ```
/// # use bs_rust::*;
/// let opts = ProxyOpts::new("example.com", "127.0.0.1:8000");
/// assert_eq!(opts.target, "example.com".to_string());
/// ```
///
#[derive(Clone, Debug)]
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
pub fn proxy_transform(_req: &HttpRequest<ProxyOpts>) -> Box<Future<Item=HttpResponse, Error=Error>> {

    // building up the new request that we'll send to the backend
    let mut outgoing = client::ClientRequest::build_from(_req);

    // build up the next outgoing URL (for the back-end)
    let next_url = format!("{}://{}{}{}",
                           match _req.uri().scheme_part() {
                               Some(scheme) => scheme.as_str(),
                               None => "http"
                           },
                           _req.state().target.clone(),
                           _req.path(),
                           match _req.uri().query().as_ref() {
                               Some(q) => format!("?{}", q),
                               None => "".to_string()
                           });

    // reset the uri so that it points to the correct proxied host + path
    outgoing.uri(next_url.as_str());

    // ensure the 'host' header is re-written
    outgoing.set_header(http::header::HOST, _req.state().target.clone());

    // now choose how to handle it
    // if the client responds with a request we want to alter (such as HTML)
    // then we need to buffer the body into memory in order to apply regex's on the string
    let next_target = _req.state().target.clone();
    let next_host = _req.uri().clone();

    outgoing
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |resp| {

            // Should we rewrite this response?
            // just check for the correct content-type header for now.
            // This will need fleshing out to provide stricter checks
            let rewrite_response = match resp.headers().get(header::CONTENT_TYPE) {
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
                    resp.body()
                        .limit(1_000_000)
                        .from_err()
                        .and_then(move |body| {

                            use std::str;

                            // In here, we now have a ful buffered response body
                            // so we can go ahead and apply URL replacements
                            let req_host = next_host.host().unwrap_or("");
                            let req_port = next_host.port().unwrap_or(80);
                            let next_body = replace_host(
                                str::from_utf8(&body[..]).unwrap(),
                                &next_target,
                                req_host, req_port,
                            );
                            let as_string = next_body.to_string();
                            Ok(create_outgoing(&resp).body(as_string))
                        })
                )
            } else {
                // If we get here, we decided not to re-write the response
                // so we just stream it back to the client
                Either::B(
                    ok(create_outgoing(&resp).body(Body::Streaming(Box::new(resp.payload().from_err()))))
                )
            }
        }).responder()
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

#[test]
fn test_forwards_headers() {
    let server = test::TestServer::new(|app| {
        app.handler(|req: &HttpRequest| {
            println!("headers received at proxy addr: {:#?}", req.headers());
            assert_eq!(req.headers().get(header::ACCEPT).unwrap(), "text/html");
            HttpResponse::Ok()
                .header("shane", "kittens")
                .header(header::CONTENT_TYPE, TEXT_HTML)
                .body(STR)
        })
    });

    let srv_address = server.addr().to_string();
    println!("orig address = {}", srv_address);

    let mut proxy = test::TestServer::build_with_state(move || {
        let addr = srv_address.clone();
        ProxyOpts::new(addr.clone())
    })
        .start(move |app| {
            app.handler(proxy_transform);
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

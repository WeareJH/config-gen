use actix_web::{
    client, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev
};
use actix::run;
use actix_web::http::header;
use futures::{Future, Stream};
use futures::future::{Either, ok};
use rewrites::replace_host;
use options::ProxyOpts;
use std::str;
use actix_web::http::HeaderMap;
use regex::Regex;
use regex::Captures;
use http::header::HeaderValue;
use actix_web::client::ClientRequest;
use actix_web::http::Cookie;
use headers::clone_headers;
use rewrites::replace_cookie_domain_on_page;

///
/// This function will clone incoming requests
/// and pass them onto a backend specified via the `target` field on [ProxyOpts]
///
pub fn proxy_transform(_req: &HttpRequest<ProxyOpts>) -> Box<Future<Item=HttpResponse, Error=Error>> {

    let req_headers = _req.headers().clone();
    let joined_cookie = req_headers.get_all(header::COOKIE).iter().map(|hdr| {
        let s = str::from_utf8(hdr.as_bytes()).unwrap_or("");
        s.to_string()
    }).collect::<Vec<String>>().join("; ");
    let next_host = _req.uri().clone();
    let req_host = next_host.host().unwrap_or("");
    let req_port = next_host.port().unwrap_or(80);
    let req_target = format!("{}:{}", req_host, req_port);
    let cloned = clone_headers(&req_headers, req_target, _req.state().target.clone());

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

    // now choose how to handle it
    // if the client responds with a request we want to alter (such as HTML)
    // then we need to buffer the body into memory in order to apply regex's on the string
    let next_target = _req.state().target.clone();
    let next_host = _req.uri().clone();
    let original_method = _req.method().as_str().clone();

    let mut outgoing = client::ClientRequest::build();
    outgoing.method(_req.method().clone()).uri(next_url);

    for (key, value) in cloned.iter() {
        outgoing.header(key.clone(), value.clone());
    }

    // ensure the 'host' header is re-written
    outgoing.set_header(http::header::HOST, _req.state().target.clone());

    // ensure the origin header is set
    outgoing.set_header(http::header::ORIGIN, _req.state().target.clone());

    outgoing.set_header(http::header::COOKIE, joined_cookie);

    if original_method == "POST" {
        let outgoing = _req.body()
            .from_err()
            .and_then(move |incoming_body| {
                outgoing.body(incoming_body).unwrap().send().map_err(Error::from)
                    .and_then(move |proxy_response| {
                        let req_host = next_host.host().unwrap_or("");
                        let req_port = next_host.port().unwrap_or(80);
                        let req_target = format!("{}:{}", req_host, req_host);
                        proxy_response.body()
                            .from_err()
                            .and_then(move |proxy_response_body| {
                                Ok(create_outgoing(
                                    &proxy_response.headers(),
                                    next_target.to_string(),
                                    req_target
                                ).body(proxy_response_body))
                            })
                    })
            });

        Box::new(outgoing)
    } else {
        outgoing.finish().unwrap().send().map_err(Error::from)
            .and_then(move |proxy_response| {

//                println!("resp from proxy {:?}", &proxy_response);

                // Should we rewrite this response?
                // just check for the correct content-type header for now.
                // This will need fleshing out to provide stricter checks
                let rewrite_response = match proxy_response.headers().get(header::CONTENT_TYPE) {
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
                        proxy_response.body()
                            .from_err()
                            .and_then(move |body| {
                                use std::str;

                                // In here, we now have a ful buffered response body
                                // so we can go ahead and apply URL replacements
                                let req_host = next_host.host().unwrap_or("");
                                let req_port = next_host.port().unwrap_or(80);
                                let req_target = format!("{}:{}", req_host, req_host);
                                let next_body = replace_host(
                                    str::from_utf8(&body[..]).unwrap(),
                                    &next_target,
                                    req_host, req_port,
                                );
                                let next_body = replace_cookie_domain_on_page(&next_body, &next_target);
                                let as_string = next_body.to_string();
                                Ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(as_string))
                            })
                    )
                } else {
                    let req_host = next_host.host().unwrap_or("");
                    let req_port = next_host.port().unwrap_or(80);
                    let req_target = format!("{}:{}", req_host, req_host);
                    // If we get here, we decided not to re-write the response
                    // so we just stream it back to the client
                    Either::B(
                        ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(Body::Streaming(Box::new(proxy_response.payload().from_err()))))
                    )
                }
            })
            .responder()
    }
}

fn create_outgoing(resp_headers: &HeaderMap, target: String, replacer: String) -> dev::HttpResponseBuilder {
    let mut outgoing = HttpResponse::Ok();
    let c = clone_headers(resp_headers, target, replacer);
    // Copy headers from backend response to main response
    for (key, value) in c.iter() {
        outgoing.header(key.clone(), value.clone());
    }
    outgoing
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use mime::TEXT_HTML;
    use actix_web::http::Cookie;
    use actix_web::http::Method;

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
        let srv_address2 = server.addr().to_string();

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            ProxyOpts::new(addr.clone())
        })
            .start(move |app| {
                app.handler(proxy_transform);
            });

        let request = proxy.get()
            .header(header::ACCEPT, "text/html")
            .set_header(header::ORIGIN, format!("https://{}", proxy.addr().to_string()))
            .uri(proxy.url("/"))
            .finish()
            .unwrap();

        let response = proxy.execute(request.send()).unwrap();
        let _bytes = proxy.execute(response.body()).unwrap();

        println!("main resp: {:#?}", response.headers());
        println!("bytes={:#?}", _bytes);

        let has_header = response.headers().get("shane").is_some();

        assert_eq!(has_header, true);
    }

    #[test]
    fn test_forwards_post_requests() {
        use bytes::Bytes;

        let server = test::TestServer::new(|app| {
            app.handler(|req: &HttpRequest| {
                println!("headers received at proxy addr: {:#?}", req.headers());
                println!("method received at proxy: {:#?}", req.method());
                assert_eq!(req.headers().get(header::ACCEPT).unwrap(), "text/html");
                req.body()
                    .and_then(move |bytes: Bytes| {
                        Ok(
                            HttpResponse::Ok()
                                .header(header::CONTENT_TYPE, "application/json")
                                .header(header::SET_COOKIE, "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.neomorganics.com")
                                .body(format!("REC-->{}", str::from_utf8(&bytes[..]).unwrap().to_string()))
                        )
                    })
                    .responder()
            })
        });

        let srv_address = server.addr().to_string();
        let srv_address2 = server.addr().to_string();

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            ProxyOpts::new(addr.clone())
        })
            .start(move |app| {
                app.handler(proxy_transform);
            });

        let request = proxy.post()
            .uri(proxy.url("/"))
            .header(header::ACCEPT, "text/html")
            .body(r#"{"hello": "world"}"#)
            .unwrap();

        let response = proxy.execute(request.send()).unwrap();
        let _bytes = proxy.execute(response.body()).unwrap();

        println!("main resp: {:#?}", response.headers());
        println!("bytes={:#?}", _bytes);

        assert_eq!(_bytes, r#"REC-->{"hello": "world"}"#)
    }

    #[test]
    fn test_strip_domain_from_cookies() {
        let cookie_value = "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.neomorganics.com";
        let cookie = Cookie::build("form_key", "40je6TqaB2SDRBeV")
            .domain("www.neomorganics.com")
            .finish();
        println!("{}", cookie);
        let mut parsed = Cookie::parse(cookie_value).unwrap();
        parsed.set_domain("");
        println!("{}", parsed.to_string());
    }
}

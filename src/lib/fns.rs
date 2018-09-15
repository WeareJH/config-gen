use actix_web::http::header;
use actix_web::http::HeaderMap;
use actix_web::{client, dev, http, Error, HttpMessage, HttpRequest, HttpResponse};
use futures::Future;
use headers::clone_headers;
use preset::AppState;
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
    let original_req_headers = original_request.headers().clone();
    let next_host = original_request.uri().clone();
    let req_host = next_host.host().unwrap_or("");
    let req_port = next_host.port().unwrap_or(80);
    let req_target = format!("{}:{}", req_host, req_port);
    let cloned = clone_headers(
        &original_req_headers,
        req_target,
        original_request.state().opts.target.clone(),
    );

    // build up the next outgoing URL (for the back-end)
    let next_url = format!(
        "{}://{}{}{}",
        match original_request.uri().scheme_part() {
            Some(scheme) => scheme.as_str(),
            None => "http",
        },
        original_request.state().opts.target.clone(),
        original_request.path(),
        match original_request.uri().query().as_ref() {
            Some(q) => format!("?{}", q),
            None => "".to_string(),
        }
    );

    // now choose how to handle it
    // if the client responds with a request we want to alter (such as HTML)
    // then we need to buffer the body into memory in order to apply regex's on the string
    let original_method = original_request.method().as_str().clone();

    let mut outgoing = client::ClientRequest::build();
    outgoing
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

    // ensure the origin header is set
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

    match original_method {
        "POST" => forward_request_with_body(original_request, outgoing),
        _ => forward_request_without_body(original_request, outgoing),
    }
}

pub fn create_outgoing(
    resp_headers: &HeaderMap,
    target: String,
    replacer: String,
) -> dev::HttpResponseBuilder {
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
    use actix_web::http::Cookie;
    use actix_web::test;
    use mime::TEXT_HTML;
    use options::ProxyOpts;

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

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            let opts = ProxyOpts::new(addr.clone());
            AppState {
                opts,
                ..Default::default()
            }
        }).start(move |app| {
            app.handler(proxy_transform);
        });

        let request = proxy
            .get()
            .header(header::ACCEPT, "text/html")
            .set_header(
                header::ORIGIN,
                format!("https://{}", proxy.addr().to_string()),
            ).uri(proxy.url("/"))
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
        use actix_web::AsyncResponder;
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
                                .header(header::SET_COOKIE, "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.acme.com")
                                .body(format!("REC-->{}", str::from_utf8(&bytes[..]).unwrap().to_string()))
                        )
                    })
                    .responder()
            })
        });

        let srv_address = server.addr().to_string();

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            let opts = ProxyOpts::new(addr.clone());
            AppState {
                opts,
                ..Default::default()
            }
        }).start(move |app| {
            app.handler(proxy_transform);
        });

        let request = proxy
            .post()
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
        let cookie_value = "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.acme.com";
        let cookie = Cookie::build("form_key", "40je6TqaB2SDRBeV")
            .domain("www.acme.com")
            .finish();
        println!("{}", cookie);
        let mut parsed = Cookie::parse(cookie_value).unwrap();
        parsed.set_domain("");
        println!("{}", parsed.to_string());
    }
}

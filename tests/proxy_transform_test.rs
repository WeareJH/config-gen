extern crate actix_web;
extern crate bs;
extern crate mime;

//#[macro_use]
extern crate env_logger;
extern crate log;

use actix_web::http::header;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use bs::proxy_transform::proxy_transform;
use bs::test_utils::get_resp;
use bs::test_utils::get_test_proxy;
use bs::test_utils::get_test_server;
use mime::{TEXT_HTML, TEXT_HTML_UTF_8};

fn test_str(adr: impl Into<String>) -> String {
    format!(
        r#"
    <!doctype html>
    <html lang="en">
    <head>
    <meta charset="UTF-8">
    </head>
    <body>
        <a href="http://{}"></a>
    </body>
    </html>
        "#,
        adr.into()
    )
}

#[test]
fn test_replace_links() {
    let (target, target_addr) = get_test_server(|app| {
        app.handler(|req: &HttpRequest| {
            let srv_address = req
                .headers()
                .get("srv_address")
                .expect("missing srv_address header")
                .to_str()
                .expect("headervalue -> str");

            HttpResponse::Ok()
                .header(header::CONTENT_TYPE, TEXT_HTML_UTF_8)
                .body(test_str(srv_address))
        });
    });

    let (mut proxy, proxy_address) = get_test_proxy(&target, |app| {
        app.handler(proxy_transform);
    });

    let request = proxy
        .get()
        .header(header::ACCEPT, TEXT_HTML)
        .header("srv_address", target_addr)
        .uri(proxy.url("/"))
        .set_header(header::HOST, proxy_address.clone())
        .finish()
        .expect("finish request");

    let (.., resp_body) = get_resp(&mut proxy, request);

    let expected_body = test_str(proxy_address.clone());

    assert_eq!(resp_body, expected_body);
}

#[test]
fn test_redirect() {
    let (target, target_addr) = get_test_server(|app| {
        app.handler(|req: &HttpRequest| {
            let srv_address = req
                .headers()
                .get("srv_address")
                .expect("missing srv_address header")
                .to_str()
                .expect("headervalue -> str");

            HttpResponse::Found()
                .header(header::LOCATION, format!("http://{}/login", srv_address))
                .finish()
        });
    });

    let (mut proxy, _proxy_address) = get_test_proxy(&target, |app| {
        app.handler(proxy_transform);
    });

    let request = proxy
        .get()
        .header(header::ACCEPT, TEXT_HTML)
        .header("srv_address", target_addr)
        .uri(proxy.url("/"))
        .finish()
        .expect("finish request");

    let (resp, _resp_body) = get_resp(&mut proxy, request);
    assert_eq!(resp.status(), 302);

    //    let _actual_redirect = resp
    //        .headers()
    //        .get(header::LOCATION)
    //        .expect("has location header")
    //        .to_str()
    //        .expect("header->str");
    //    let expected_redirect = format!("http://{}/login", proxy_address);
    //    assert_eq!(actual_redirect, expected_redirect);
}

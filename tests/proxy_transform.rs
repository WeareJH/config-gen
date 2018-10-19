extern crate actix_web;
extern crate bs;
extern crate mime;

#[macro_use]
extern crate log;
extern crate env_logger;

use actix_web::http::header;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{test, HttpResponse};
use bs::config::ProgramConfig;
use bs::options::ProgramOptions;
use bs::preset::AppState;
use bs::presets::m2::requirejs_config::RequireJsClientConfig;
use bs::proxy_transform::proxy_transform;
use mime::{TEXT_HTML, TEXT_HTML_UTF_8};
use std::str;
use std::sync::Arc;
use std::sync::Mutex;

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
fn test_forwards_headers() {
    let server = test::TestServer::new(|app| {
        app.handler(|req: &HttpRequest| {
            debug!("headers received at proxy addr: {:#?}", req.headers());
            assert_eq!(
                req.headers()
                    .get(header::ACCEPT)
                    .expect("has accept header"),
                "text/html"
            );
            assert_eq!(
                req.headers()
                    .get(header::COOKIE)
                    .expect("has cookie header"),
                "hello there; hello there 2"
            );

            let srv_address = req
                .headers()
                .get("srv_address")
                .expect("missing srv_address header")
                .to_str()
                .expect("headervalue -> str");

            HttpResponse::Ok()
                .header("shane", "kittens")
                .header(header::CONTENT_TYPE, TEXT_HTML_UTF_8)
                .body(test_str(srv_address))
        })
    });

    let srv_address = server.addr().to_string();
    let srv_address2 = srv_address.clone();

    let mut proxy = test::TestServer::build_with_state(move || {
        let addr = srv_address.clone();
        let opts = ProgramOptions::new(addr.clone(), "http");
        AppState {
            opts,
            program_config: ProgramConfig::default(),
            rewrites: vec![],
            req_log: Mutex::new(vec![]),
            rjs_client_config: Arc::new(Mutex::new(RequireJsClientConfig::default())),
        }
    }).start(move |app| {
        app.handler(proxy_transform);
    });

    debug!("PROXY={}", proxy.addr().to_string());
    debug!("TARGET={}", srv_address2.clone());

    let request = proxy
        .get()
        .header(header::ACCEPT, TEXT_HTML)
        .header("cookie", "hello there")
        .header("cookie", "hello there 2")
        .header("srv_address", srv_address2)
        .set_header(
            header::ORIGIN,
            format!("https://{}", proxy.addr().to_string()),
        ).uri(proxy.url("/"))
        .set_header(header::HOST, proxy.addr().to_string())
        .finish()
        .unwrap();

    let response = proxy.execute(request.send()).unwrap();
    let _bytes = proxy.execute(response.body()).unwrap();
    let response_body = str::from_utf8(&_bytes[..])
        .expect("bytes->String")
        .to_string();
    let expected_body = test_str(proxy.addr().to_string());

    debug!("main resp headers: {:#?}", response.headers());

    let has_header = response.headers().get("shane").is_some();

    assert_eq!(has_header, true);
    assert_eq!(response_body, expected_body);
}

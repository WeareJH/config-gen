#![allow(unused_imports)]
#![allow(dead_code)]
extern crate actix_web;
extern crate bytes;
extern crate futures;
extern crate bs_rust;

use actix_web::{HttpRequest, HttpResponse, Error, client, test, HttpMessage, AsyncResponder};
use bytes::Bytes;
use futures::Future;
use std::sync::Arc;
use std::net::SocketAddr;
use actix_web::http::header;
use bs_rust::{ProxyOpts, proxy_transform};

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

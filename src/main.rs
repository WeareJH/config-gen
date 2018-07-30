#![allow(unused_imports)]
#![allow(dead_code)]
extern crate actix_web;
extern crate bytes;
extern crate futures;

use actix_web::{HttpRequest, HttpResponse, Error, client, test, HttpMessage, AsyncResponder};
use bytes::{Bytes};
use futures::{Future};
use std::sync::Arc;
use std::net::SocketAddr;
use actix_web::http::header;

fn main() {}

const STR: &str = "Hello world";

#[test]
fn test_body() {
    let server = test::TestServer::new(|app| app.handler(|req: &HttpRequest| {
        println!("headers received at proxy addr: {:#?}", req.headers());
        HttpResponse::Ok()
            .header("shane", "kittens")
            .body(STR)
    }));

    let srv_address = server.addr().to_string();
    println!("orig address = {}", srv_address);

    let mut proxy = test::TestServer::new(move |app| {

        let addr = srv_address.clone();

        app.handler(move |_req: &HttpRequest| -> Box<Future<Item = HttpResponse, Error = Error>> {
            println!("proxy handler: {:#?}", _req.headers());

            let mut outgoing = client::ClientRequest::get(format!("http://{}", addr.clone()).as_str());

            for (key, value) in _req.headers() {
                outgoing.header(key.clone(), value.clone());
            }

            outgoing.set_header(header::HOST, addr.clone());

            outgoing
                .finish()
                .unwrap()
                .send()
                .map_err(Error::from)          // <- convert SendRequestError to an Error
                .and_then(|resp| {
                    println!("proxy RESP: {:#?}", resp.headers());
                    resp.body()         // <- this is MessageBody type, resolves to complete body
                        .from_err()            // <- convert PayloadError to an Error
                        .and_then(move |body| {     // <- we got complete body, now send as server response
                            let mut outgoing = HttpResponse::Ok();
                            for (key, value) in resp.headers() {
                                outgoing.header(key.clone(), value.clone());
                            }
                            Ok(outgoing.body(body))
                        })
                })
                .responder()
        });
    });

    let request = proxy.get()
        .uri(proxy.url("/"))
        .finish()
        .unwrap();

    let response = proxy.execute(request.send()).unwrap();
    let _bytes = proxy.execute(response.body()).unwrap();

    let has_header = response.headers().get("shane").unwrap();

    assert_eq!(has_header, "kittens");
}

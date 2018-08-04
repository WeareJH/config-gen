use actix_web::{
    client, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev, test
};
use futures::{Future, Stream};
use actix_web::Binary;
use futures::future::{Either, ok};
use fns::proxy_transform;
use fns::ProxyOpts;

fn index(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    client::ClientRequest::get("https://www.rust-lang.org/en-US/")
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            // based on some_condition, either buffer and re-write or 'pass-through'
            if true {
                Either::A(
                    resp.body()
                        .from_err()
                        .and_then(|body| {
                            Ok(HttpResponse::Ok().body(body))
                        })
                )
            } else {
                Either::B(
                    ok(HttpResponse::Ok().body(Body::Streaming(Box::new(resp.payload().from_err()))))
                )
            }
        })
        .responder()
}



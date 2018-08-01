#![allow(unused_variables)]
extern crate actix;
extern crate env_logger;
extern crate futures;
extern crate actix_web;
extern crate openssl;
extern crate url;

use actix_web::{middleware, server, App};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod lib;
use lib::proxy_transform;
use lib::ProxyOpts;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("http-proxy");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("src/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("src/cert.pem").unwrap();

    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .default_resource(|r| r.f(move |req| proxy_transform(req, ProxyOpts::new("www.neomorganics.com"))))
    }).bind_ssl("127.0.0.1:8080", builder)
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
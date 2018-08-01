extern crate actix;
extern crate env_logger;
extern crate futures;
extern crate actix_web;

use actix_web::{server, App, middleware};
use lib::proxy_transform;
use lib::ProxyOpts;

mod lib;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("http-proxy");

    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .default_resource(|r| r.f(move |req| proxy_transform(req, ProxyOpts::new("yourmomentscaptured.co.uk"))))
    }).workers(1)
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
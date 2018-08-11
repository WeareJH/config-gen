#![allow(unused_variables)]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate openssl;
extern crate url;
extern crate regex;
extern crate mime;
extern crate clap;
extern crate bytes;
extern crate http;

use actix_web::{middleware, server, App};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod fns;
mod test;
mod headers;
mod rewrites;
mod options;
use fns::proxy_transform;
use options::ProxyOpts;
use clap::App as ClapApp;
use clap::Arg;
use options::get_host;

fn main() {

    let matches = ClapApp::new("bs-rust")
        .arg(Arg::with_name("input").required(true))
        .arg(Arg::with_name("port").short("p").long("port").takes_value(true))
        .get_matches();

    match get_host(matches.value_of("input").unwrap_or("")) {
        Ok(host) => {
            let opts = ProxyOpts::new(host);
            run(opts);
        },
        Err(err) => println!("{}", err)
    }
}

fn run(opts: ProxyOpts) {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("https-proxy");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder
        .set_private_key_file("src/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("src/cert.pem").unwrap();

    let local_addr = format!("127.0.0.1:{}", opts.port);

    server::new(move || {
        App::with_state(opts.clone())
//            .middleware(middleware::Logger::default())
            .default_resource(|r| r.f(proxy_transform))
    }).bind_ssl(&local_addr, builder)
        .unwrap()
        .start();

    println!("Started https server: {}", local_addr);
    let _ = sys.run();
}

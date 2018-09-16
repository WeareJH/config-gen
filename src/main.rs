#![allow(unused_variables)]
#![allow(unused_imports)]
extern crate actix;
extern crate actix_web;
extern crate bs;
extern crate bytes;
extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate mime;
extern crate openssl;
extern crate regex;
extern crate url;

use actix_web::{server, App};
use clap::App as ClapApp;
use clap::Arg;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use bs::fns::proxy_transform;
use bs::options::{get_host, ProxyOpts};
use bs::preset::AppState;
use bs::preset::Preset;
use bs::preset_m2::M2Preset;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    let matches = ClapApp::new("bs-rust")
        .arg(Arg::with_name("input").required(true))
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        ).get_matches();

    match get_host(matches.value_of("input").unwrap_or("")) {
        Ok(host) => {
            let opts = ProxyOpts::new(host)
                .with_port(matches.value_of("port").unwrap_or("8080").parse().unwrap());
            run(opts);
        }
        Err(err) => println!("{}", err),
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
        let preset = M2Preset::new();

        // AppState is available to all Middleware
        // and response handlers
        let app_state = AppState {
            opts: opts.clone(),
            rewrites: preset.rewrites(),
            module_items: Mutex::new(vec![]),
        };

        // add initial state & middleware
        let app = App::with_state(app_state);

        let app = preset.add_before_middleware(app);

        // Enhance the app by allowing this preset to add middleware
        // or resources
        let app = preset.enhance(app);

        // now add the default response type
        let app = app.default_resource(|r| r.f(proxy_transform));

        // finally return the App
        app
    }).workers(1)
    .bind_ssl(&local_addr, builder)
    .unwrap()
    .start();

    println!("Started https server: https://{}", local_addr);
    let _ = sys.run();
}

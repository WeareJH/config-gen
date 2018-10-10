#![allow(unused_variables)]
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
extern crate serde_yaml;
extern crate url;

use actix_web::{server, App};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use bs::config::{ProgramConfig, ProgramStartError};
use bs::from_file::FromFile;
use bs::options::{ProgramOptions, ProxyScheme};
use bs::setup::{apply_presets, state_and_presets};
use openssl::ssl::SslAcceptorBuilder;

fn main() {
    match ProgramOptions::from_vec(&mut std::env::args_os()).and_then(run_with_opts) {
        Ok(opts) => println!("Running!"),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn run_with_opts(opts: ProgramOptions) -> Result<(), ProgramStartError> {
    //
    // Logging config
    //
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    //
    // The underlying Actor System
    //
    let sys = actix::System::new("https-proxy");

    //
    // Get program configuration, from the input above, and
    // then eventuall from a file
    //
    let file_path = opts
        .config_file
        .clone()
        .expect("config_file cannot be missing");

    //
    // Pull the ProgramConfig from a  file
    //
    let program_config =
        ProgramConfig::from_yml_file(&file_path).map_err(|e| ProgramStartError::FromFile(e))?;

    //
    // Clone server opts to be used in multi threads
    //
    let server_opts = opts.clone();

    //
    // The address that the server will be accessible on
    //
    let local_addr = format!("127.0.0.1:{}", opts.port.clone());

    //
    // Did the user provide a seed?
    //
    let maybe_seed = server_opts.seed_file.clone();

    //
    // Now start the server
    //
    let s = server::new(move || {
        let (app_state, presets_map) = state_and_presets(&opts, &program_config, &maybe_seed);
        let app = App::with_state(app_state);
        apply_presets(app, &program_config, &presets_map)
    }).workers(1);

    //
    // Bind on either http or https depending on the
    // target URL's scheme
    //
    let s = match server_opts.scheme {
        ProxyScheme::Http => s.bind(&local_addr),
        ProxyScheme::Https => s.bind_ssl(&local_addr, get_ssl_builder()),
    };

    s.expect("Couldn't start the application").start();

    println!("Started server: {}://{}", server_opts.scheme, local_addr);

    let _ = sys.run();

    Ok(())
}

///
/// SSL builder
///
/// Todo: allow key/cert options
///
fn get_ssl_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("src/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("src/cert.pem").unwrap();
    builder
}

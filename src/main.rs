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
extern crate tempdir;
extern crate url;

use actix_web::{server, App};

use bs::config::{ProgramConfig, ProgramStartError};
use bs::from_file::FromFile;
use bs::options::{ProgramOptions, ProxyScheme};
use bs::setup::{apply_presets, state_and_presets};
use bs::ssl;

fn main() {
    match ProgramOptions::from_vec(&mut std::env::args_os()).and_then(run_with_opts) {
        Ok(..) => { /* Running! */ }
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
        ProxyScheme::Http => s.bind(&local_addr).map_err(ProgramStartError::BindHttp)?,
        ProxyScheme::Https => {
            let builder = ssl::builder()?;
            s.bind_ssl(&local_addr, builder)
                .map_err(ProgramStartError::BindHttps)?
        }
    };

    //
    // Start the server
    //
    s.shutdown_timeout(0).start();

    //
    // Output the proxy URL only
    //
    println!("{}://{}", server_opts.scheme, local_addr);

    let _ = sys.run();

    Ok(())
}

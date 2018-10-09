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
use bs::options::ProgramOptions;
use bs::options::ProxyScheme;
use bs::preset::{AppState, Preset};
use bs::presets::m2::opts::M2PresetOptions;
use bs::presets::m2::preset_m2::M2Preset;
use bs::presets::m2::requirejs_config::RequireJsClientConfig;
use bs::presets::m2::seed::SeedData;
use bs::proxy_transform::proxy_transform;
use openssl::ssl::SslAcceptorBuilder;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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
    // The address that the server will be accessible on
    //
    let local_addr = format!("127.0.0.1:{}", opts.port);

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

    let maybe_seed = server_opts.seed_file.clone();

    //
    // Now start the server
    //
    let s = server::new(move || {
        //
        // Use a HashMap + index lookup for anything
        // that implements Preset
        //
        let mut presets_map: HashMap<usize, Box<Preset<AppState>>> = HashMap::new();

        let mut app_state = create_state(maybe_seed.clone(), program_config.clone(), opts.clone());

        //
        // Loop through any presets and create an instance
        // that's stored in the hashmap based on it's index
        //
        // This is done so that we can use the index later
        // to lookup this item in order
        //
        for (index, p) in program_config.presets.iter().enumerate() {
            match p.name.as_str() {
                "m2" => {
                    let preset_opts: M2PresetOptions =
                        serde_yaml::from_value(p.options.clone()).unwrap();
                    let preset = M2Preset::new(preset_opts);
                    presets_map.insert(index, Box::new(preset));
                }
                _ => println!("unsupported"),
            }
        }

        // Add rewrites phase
        for (index, _) in program_config.presets.iter().enumerate() {
            let subject_preset = presets_map.get(&index).expect("Missing preset");
            app_state.rewrites.extend(subject_preset.rewrites());
        }

        let mut app = App::with_state(app_state);

        // before middlewares
        for (index, _) in program_config.presets.iter().enumerate() {
            let subject_preset = presets_map.get(&index).expect("Missing preset");
            app = subject_preset.add_before_middleware(app);
        }

        // enhances
        for (index, _) in program_config.presets.iter().enumerate() {
            let subject_preset = presets_map.get(&index).expect("Missing preset");
            app = subject_preset.enhance(app);
        }

        let app = app.default_resource(|r| r.f(proxy_transform));

        // finally return the App
        app
    }).workers(1);

    let s = match server_opts.scheme {
        ProxyScheme::Http => s.bind(&local_addr),
        ProxyScheme::Https => s.bind_ssl(&local_addr, get_ssl_builder()),
    };

    s.expect("Couldn't bind").start();

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

///
/// Build up the application state based on a potential
/// incoming seed
///
pub fn create_state(
    maybe_seed: Option<String>,
    program_config: ProgramConfig,
    opts: ProgramOptions,
) -> AppState {
    let (req_log, rjs_client_config) = match maybe_seed {
        Some(ref s) => match SeedData::from_json_file(&s) {
            Ok(seed) => (seed.req_log, seed.rjs_client_config),
            Err(e) => {
                eprintln!("Could not read seed, {:?}", e);
                (vec![], RequireJsClientConfig::default())
            }
        },
        None => (vec![], RequireJsClientConfig::default()),
    };

    AppState {
        program_config,
        opts,
        rewrites: vec![],
        req_log: Mutex::new(req_log),
        rjs_client_config: Arc::new(Mutex::new(rjs_client_config)),
    }
}

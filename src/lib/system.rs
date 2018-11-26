use actix;
use actix_web::{server, App};
use config::{ProgramConfig, ProgramStartError};
use from_file::FromFile;
use options::ProgramOptions;
use options::ProxyScheme;
use setup::apply_presets;
use setup::state_and_presets;
use setup::validate_presets;
use ssl;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

pub fn create(opts: ProgramOptions) -> Result<(actix::SystemRunner, String), ProgramStartError> {
    //
    // The underlying Actor System
    //
    let sys = actix::System::new("https-proxy");

    //
    // Pull the ProgramConfig from a  file
    //
    let program_config = match opts.config_file.clone() {
        Some(cfg_path) => {
            ProgramConfig::from_file(&cfg_path).map_err(|e| ProgramStartError::FromFile(e))?
        }
        None => ProgramConfig::default_preset(),
    };

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
    // Exit early if any presets fail validation
    //
    let _validated_presets = validate_presets(&program_config)?;

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
    // Get the first address that was bound successfully
    //
    let addrs = s.addrs();
    let first_addr = match addrs.get(0) {
        Some(SocketAddr::V4(addr)) => Some(addr),
        Some(SocketAddr::V6(..)) => None,
        None => None,
    };

    //
    // if there's not at LEAST 1 address, it's a program start error
    //
    let addr: &SocketAddrV4 = first_addr.ok_or(ProgramStartError::Ip)?;

    //
    // Start the server
    //
    s.shutdown_timeout(0).start();

    let output_addr = format!("{}://{}:{}", server_opts.scheme, addr.ip(), addr.port());

    Ok((sys, output_addr))
}

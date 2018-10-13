use actix;
use actix_web::{server, App};
use config::{ProgramConfig, ProgramStartError};
use from_file::FromFile;
use options::ProgramOptions;
use options::ProxyScheme;
use setup::apply_presets;
use setup::state_and_presets;
use ssl;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

pub fn create(opts: ProgramOptions) -> Result<(actix::SystemRunner, String), ProgramStartError> {
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

    Ok((
        sys,
        format!("{}://{}:{}", server_opts.scheme, addr.ip(), addr.port()),
    ))
}

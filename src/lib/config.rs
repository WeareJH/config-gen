extern crate serde_yaml;

use clap::App as ClapApp;
use clap::Arg;
use options::get_host;
use options::ConfigError;
use options::ProxyOpts;
use serde_yaml::Value;
use std::fmt::Formatter;

#[derive(Deserialize, Debug)]
pub struct PresetConfig {
    pub name: String,
    pub options: Value,
}

#[derive(Deserialize, Debug)]
pub struct ProgramConfig {
    pub presets: Vec<PresetConfig>,
}

pub fn get_program_config_from_string(input: &str) -> Result<ProgramConfig, serde_yaml::Error> {
    serde_yaml::from_str(input)
}

pub enum ProgramStartError {
    ConfigParseError(serde_yaml::Error),
    ConfigCliError(ConfigError),
}

impl std::fmt::Display for ProgramStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProgramStartError::ConfigParseError(e) => write!(f, "could not parse config"),
            ProgramStartError::ConfigCliError(e) => {
                write!(f, "could not parse incoming options from CLI")
            }
        }
    }
}

///
/// Options that come in via the CLI flags
///
pub fn get_program_config_from_cli() -> Result<ProxyOpts, ProgramStartError> {
    let matches = ClapApp::new("bs-rust")
        .arg(Arg::with_name("input").required(true))
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true),
        ).get_matches();

    match get_host(matches.value_of("input").unwrap_or("")) {
        Ok((host, scheme)) => Ok(ProxyOpts::new(host, scheme)
            .with_port(matches.value_of("port").unwrap_or("8080").parse().unwrap())),
        Err(err) => Err(ProgramStartError::ConfigCliError(err)),
    }
}

extern crate serde_yaml;

use preset_m2_config_gen::BundleConfig;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub enum ConfigType {
    Invalid,
    File(String),
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidInput(String),
    FileOpen,
    FileRead,
    SerdeError(serde_yaml::Error),
}

pub fn resolve_from_string(input: String) -> Result<BundleConfig, ConfigError> {
    let conf = get_config(&input);

    match conf {
        Err(config_error) => match config_error {
            ConfigError::SerdeError(e) => {
                eprintln!("{}", e);
                Err(ConfigError::SerdeError(e))
            }
            ConfigError::InvalidInput(input) => {
                eprintln!("{}", input);
                Err(ConfigError::InvalidInput(input))
            }
            _e => {
                eprintln!("Other error {:?}", _e);
                Err(_e)
            }
        },
        Ok(p) => Ok(p),
    }
}

///
/// From a string like `file:config.yaml`, try to read the file
/// and if it exists, parse into a strongly typed struct
///
fn get_config(input: &str) -> Result<BundleConfig, ConfigError> {
    get_file_path(&input)
        .and_then(read_from_path)
        .and_then(parse_from_string)
}

///
/// Parse strings like file:config.yaml to extract the file path only
///
fn get_file_path(input: &str) -> Result<String, ConfigError> {
    let split: Vec<&str> = input.split(":").collect();
    match split.len() {
        1 => Ok(split[0].into()),
        2 => Ok(split[1].into()),
        _ => Err(ConfigError::InvalidInput(input.to_string())),
    }
}

///
/// Take a user-given path & try to read the file from disk into a String
///
fn read_from_path(maybe_path: String) -> Result<String, ConfigError> {
    let mut file = File::open(maybe_path).map_err(|_| ConfigError::FileOpen)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| ConfigError::FileRead)?;
    Ok(contents)
}

///
/// Parse any YAML string directly into a Struct
///
fn parse_from_string(contents: String) -> Result<BundleConfig, ConfigError> {
    serde_yaml::from_str(&contents).map_err(|e| ConfigError::SerdeError(e))
}

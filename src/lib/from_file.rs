extern crate serde_json;
extern crate serde_yaml;

use serde::Deserialize;
use std;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FromFileError {
    InvalidInput,
    FileOpen(PathBuf),
    FileRead,
    SerdeError(String),
}

///
/// Implement this trait to enable your Struct's to deserialized
/// from a file-path like
///
/// - conf/app.yaml
/// - file:conf/app.yaml
///
pub trait FromFile {
    ///
    /// From a string like `file:config.yaml`, try to read the file
    /// and if it exists, parse into a strongly typed struct `Person`
    ///
    fn from_yml_file(input: &str) -> Result<Self, FromFileError>
    where
        for<'de> Self: Deserialize<'de> + Sized,
    {
        <Self as FromFile>::get_file_path(input)
            .and_then(<Self as FromFile>::file_read)
            .and_then(<Self as FromFile>::from_yaml_string)
    }

    ///
    /// From a string like `file:config.yaml`, try to read the file
    /// and if it exists, parse into a strongly typed struct `Person`
    ///
    fn from_json_file(input: &str) -> Result<Self, FromFileError>
    where
        for<'de> Self: Deserialize<'de> + Sized,
    {
        <Self as FromFile>::get_file_path(input)
            .and_then(<Self as FromFile>::file_read)
            .and_then(<Self as FromFile>::from_json_string)
    }

    ///
    /// Parse strings like file:config.yaml to extract the file path only
    ///
    fn get_file_path(input: &str) -> Result<String, FromFileError> {
        let split: Vec<&str> = input.split(":").collect();
        match split.len() {
            1 => Ok(split[0].into()),
            2 => Ok(split[1].into()),
            _ => Err(FromFileError::InvalidInput),
        }
    }

    ///
    /// Attempt to Read the file's contents into a string
    ///
    fn file_read(input: String) -> Result<String, FromFileError> {
        let mut maybe_path = std::env::current_dir().expect("can read current dir");
        maybe_path.push(&input);
        let mut file = File::open(&input).map_err(|_| FromFileError::FileOpen(maybe_path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| FromFileError::FileRead)?;
        Ok(contents)
    }

    ///
    /// Parse any YAML string directly into a Self
    ///
    fn from_yaml_string(contents: String) -> Result<Self, FromFileError>
    where
        for<'de> Self: Deserialize<'de>,
    {
        serde_yaml::from_str(&contents).map_err(|e| FromFileError::SerdeError(e.to_string()))
    }

    ///
    /// Parse json string directly into a Self
    ///
    fn from_json_string(contents: String) -> Result<Self, FromFileError>
    where
        for<'de> Self: Deserialize<'de>,
    {
        serde_json::from_str(&contents).map_err(|e| FromFileError::SerdeError(e.to_string()))
    }
}

impl std::fmt::Display for FromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FromFileError::InvalidInput => write!(f, "ConfigError::InvalidInput"),
            FromFileError::FileOpen(path) => {
                write!(f, "ConfigError::FileOpen - couldn't open {:?}", path)
            }
            FromFileError::FileRead => write!(f, "ConfigError::FileRead"),
            FromFileError::SerdeError(e) => write!(f, "ConfigError::SerdeError - {}", e),
        }
    }
}

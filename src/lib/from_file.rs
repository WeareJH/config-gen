extern crate serde_yaml;
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;
use serde::Deserialize;

#[derive(Debug)]
pub enum ConfigError {
    InvalidInput,
    FileOpen,
    FileRead,
    SerdeError(String)
}

//impl From<ConfigError> for String {
//    fn from(e: ConfigError) -> Self {
//        let output = match e {
//            ConfigError::InvalidInput => "Invalid Input",
//            ConfigError::FileOpen => "Couldn't open the file",
//            ConfigError::FileRead => "Couldn't read the file contents",
//            ConfigError::SerdeError(e) => e.to_string(),
//        };
//        output.to_string()
//    }
//}

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
    fn from_yml_file(input: &str) -> Result<Self, ConfigError>
        where for<'de> Self: Deserialize<'de> + Sized
    {
        <Self as FromFile>::get_file_path(input)
            .and_then(<Self as FromFile>::file_read)
            .and_then(<Self as FromFile>::from_yaml_string)
    }

    ///
    /// From a string like `file:config.yaml`, try to read the file
    /// and if it exists, parse into a strongly typed struct `Person`
    ///
    fn from_json_file(input: &str) -> Result<Self, ConfigError>
        where for<'de> Self: Deserialize<'de> + Sized
    {
        <Self as FromFile>::get_file_path(input)
            .and_then(<Self as FromFile>::file_read)
            .and_then(<Self as FromFile>::from_json_string)
    }

    ///
    /// Parse strings like file:config.yaml to extract the file path only
    ///
    fn get_file_path(input: &str) -> Result<String, ConfigError> {
        let split: Vec<&str> = input.split(":").collect();
        match split.len() {
            1 => Ok(split[0].into()),
            2 => Ok(split[1].into()),
            _ => Err(ConfigError::InvalidInput)
        }
    }

    ///
    /// Attempt to Read the file's contents into a string
    ///
    fn file_read(input: String) -> Result<String, ConfigError> {
        let mut file = File::open(input).map_err(|_| ConfigError::FileOpen)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|_| ConfigError::FileRead)?;
        Ok(contents)
    }

    ///
    /// Parse any YAML string directly into a Self
    ///
    fn from_yaml_string(contents: String) -> Result<Self, ConfigError>
        where for<'de> Self: Deserialize<'de>
    {
        serde_yaml::from_str(&contents).map_err(|e| ConfigError::SerdeError(e.to_string()))
    }

    ///
    /// Parse json string directly into a Self
    ///
    fn from_json_string(contents: String) -> Result<Self, ConfigError>
        where for<'de> Self: Deserialize<'de>
    {
        serde_json::from_str(&contents).map_err(|e| ConfigError::SerdeError(e.to_string()))
    }
}

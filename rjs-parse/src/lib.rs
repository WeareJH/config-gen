extern crate ratel;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod parse;
pub mod config;

pub use crate::config::RequireJsClientConfig;
pub use crate::config::RequireJsBuildConfig;
pub use crate::config::Module;

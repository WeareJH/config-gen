extern crate ratel;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod parse;
pub mod config;

pub use config::RequireJsClientConfig;
pub use config::RequireJsBuildConfig;
pub use config::Module;

extern crate ratel;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod build_config;
pub mod client_config;
pub mod parse;

pub use client_config::BuildModule;
pub use client_config::RequireJsClientConfig;

pub use build_config::RequireJsBuildConfig;

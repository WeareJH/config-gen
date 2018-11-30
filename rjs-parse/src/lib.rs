#[macro_use]
extern crate from_file_derive;
extern crate from_file;
extern crate ratel;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod build_config;
pub mod bundle_config;
pub mod client_config;
pub mod modules;
pub mod parse;

pub use client_config::RequireJsClientConfig;
pub use modules::BuildModule;

pub use build_config::RequireJsBuildConfig;
pub use modules::ModuleData;
pub use bundle_config::BundleConfig;

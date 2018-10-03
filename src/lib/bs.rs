#![allow(dead_code)]
#![feature(extern_prelude)]
extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bytes;
extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate mime;
extern crate openssl;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate url;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod config;
pub mod headers;
pub mod options;
pub mod preset;
pub mod preset_m2;
pub mod preset_m2_bundle_config;
pub mod preset_m2_config_gen;
pub mod preset_m2_opts;
pub mod preset_m2_requirejs_config;
pub mod proxy_transform;
pub mod replacer;
pub mod rewrites;
pub mod with_body;
pub mod without_body;
pub mod from_file;

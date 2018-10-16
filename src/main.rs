#![allow(unused_variables)]
extern crate actix;
extern crate actix_web;
extern crate bs;
extern crate bytes;
extern crate clap;

extern crate futures;
extern crate http;
extern crate mime;
extern crate openssl;
extern crate regex;
extern crate serde_yaml;
extern crate tempdir;
extern crate url;

#[macro_use]
extern crate log;
extern crate env_logger;

use bs::options::ProgramOptions;
use bs::system;

fn main() {
    //
    // Logging config for CLI
    //
    env_logger::init();

    match ProgramOptions::from_args(&mut std::env::args_os()).and_then(system::create) {
        Ok((sys, url)) => {
            println!("{}", url);
            let _ = sys.run();
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

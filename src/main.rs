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

extern crate env_logger;
extern crate log;

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
            println!("{}/__bs/build.json\t(Generates the configuration needed for the Optimizer)", url);
            println!("{}/__bs/loaders.js\t(Generates the JavaScript needed to load additional bundles)", url);
            println!("{}/__bs/seed.json\t(Generates a dump of the current state so that you can pick up where you left off)", url);
            let _ = sys.run();
        }
        Err(e) => {
            eprintln!("Sorry there was a problem starting config-gen");
            eprintln!("Please see the error below:\n");
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

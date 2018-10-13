extern crate bs;
extern crate actix;
extern crate actix_web;
extern crate futures;

use bs::options::ProgramOptions;
use bs::system;
use actix::System;
use actix_web::client;
use actix::Handler;
use futures::future::Future;

#[test]
fn test_add() {
    let args = vec!["config-gen", "http://example.com", "--config", "test/fixtures/config.yml"];
    match ProgramOptions::from_args(args).and_then(system::create) {
        Ok((sys, url)) => {
            println!("{}", url);

            actix::Arbiter::handle().spawn({
                client::get(url)
                    .finish().unwrap()
                    .send()
                    .map_err(|_| ())
                    .and_then(|response| {
                        println!("Response: {:?}", response);
                        System::current().stop();
                        Ok(())
                    })
            });

            System::run(|| {});
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

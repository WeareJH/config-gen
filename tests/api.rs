extern crate actix;
extern crate actix_web;
extern crate bs;
extern crate futures;
extern crate reqwest;
extern crate rjs;
extern crate serde_json;

use actix::System;
use actix_web::http;
use bs::config::ProgramStartError;
use bs::options::ProgramOptions;
use bs::presets::m2::seed::SeedData;
use bs::system;
use reqwest::Response;
use rjs::{RequireJsBuildConfig, RequireJsClientConfig};

type RunResult = Result<(actix::SystemRunner, String), ProgramStartError>;
type ApiResult = Result<(actix::SystemRunner, String, Response), ProgramStartError>;

///
/// These are some default args (as would be seen in the CLI)
///that can be re-used in tests
///
const DEFAULT_ARGS: &'static [&'static str] = &[
    "config-gen",
    "http://example.com",
    "--config",
    "test/fixtures/config.yml",
];

#[test]
fn test_config_json() {
    api_get(DEFAULT_ARGS.to_vec(), "/__bs/config.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let _c: RequireJsClientConfig =
            serde_json::from_str(&res.text().expect("unwrap text response"))
                .expect("serde deserialize");
    });
}

#[test]
fn test_loaders_js() {
    api_get(DEFAULT_ARGS.to_vec(), "/__bs/loaders.js", |result| {
        let (_sys, _url, res) = result.expect("api returned");
        let ct = &res
            .headers()
            .get(http::header::CONTENT_TYPE)
            .expect("has content-type");
        assert_eq!(ct.to_str().expect("header->str"), "application/javascript");
    });
}

#[test]
fn test_seed_json() {
    api_get(DEFAULT_ARGS.to_vec(), "/__bs/seed.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let t = res.text().expect("unwrap text response");
        let _c: SeedData = serde_json::from_str(&t).expect("serde deserialize");
    });
}

#[test]
fn test_seed_seeded_json() {
    let mut args = DEFAULT_ARGS.to_vec().clone();
    args.extend(vec!["--seed", "test/fixtures/seed.json"]);
    api_get(args, "/__bs/seed.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let t = res.text().expect("unwrap text response");
        let c: SeedData = serde_json::from_str(&t).expect("serde deserialize");
        assert_eq!(c.req_log.len(), 339);
    });
}

#[test]
fn test_build_json() {
    api_get(DEFAULT_ARGS.to_vec(), "/__bs/build.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let _c: RequireJsBuildConfig =
            serde_json::from_str(&res.text().expect("unwrap text response"))
                .expect("serde deserialize");
    });
}

#[test]
fn test_build_json_from_json_config() {
    let args = vec![
        "config-gen",
        "http://example.com",
        "--config",
        "test/fixtures/config.json",
    ];
    api_get(args, "/__bs/build.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let _c: RequireJsBuildConfig =
            serde_json::from_str(&res.text().expect("unwrap text response"))
                .expect("serde deserialize");
    });
}

#[test]
fn test_build_json_without_config() {
    let args = vec!["config-gen", "http://example.com"];
    api_get(args, "/__bs/build.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let _c: RequireJsBuildConfig =
            serde_json::from_str(&res.text().expect("unwrap text response"))
                .expect("serde deserialize");
    });
}

#[test]
fn test_build_json_with_seed_without_config() {
    let args = vec![
        "config-gen",
        "http://example.com",
        "--seed",
        "test/fixtures/seed.json",
    ];
    api_get(args, "/__bs/build.json", |result| {
        let (_sys, _url, mut res) = result.expect("api returned");
        let _c: RequireJsBuildConfig =
            serde_json::from_str(&res.text().expect("unwrap text response"))
                .expect("serde deserialize");
    });
}

#[test]
fn test_validate_preset_options() {
    let args = vec![
        "config-gen",
        "http://example.com",
        "--config",
        "test/fixtures/config-error.json",
    ];
    match ProgramOptions::from_args(args).and_then(system::create) {
        Ok(..) => {
            unreachable!();
        }
        Err(_e) => {
            assert!(true)
        }
    }
}

#[test]
fn test_exit_on_unsupported_preset() {
    let args = vec![
        "config-gen",
        "http://example.com",
        "--config",
        "test/fixtures/config-unsupported-preset.json",
    ];
    match ProgramOptions::from_args(args).and_then(system::create) {
        Ok(..) => {
            unreachable!();
        }
        Err(_e) => {
            assert!(true)
        }
    }
}

///
/// Test helper to run the server from a Vec of args
/// just like you would in the the CLI
///
/// # Examples
///
fn run_with_args<S>(args: Vec<&str>, cb: S)
where
    S: FnOnce(RunResult) + 'static,
{
    match ProgramOptions::from_args(args).and_then(system::create) {
        Ok((sys, url)) => {
            System::run(move || {
                cb(Ok((sys, url)));
                System::current().stop();
            });
        }
        Err(e) => {
            cb(Err(e));
        }
    }
}

///
/// Execute an API GET request
///
fn api_get<F>(args: Vec<&str>, path: &'static str, cb: F)
where
    F: FnOnce(ApiResult) + 'static,
{
    run_with_args(args, move |result: RunResult| {
        let (sys, url) = result.expect("system started");
        let api1 = format!("{}{}", url, path);
        let res = reqwest::get(api1.as_str()).expect("call config.json api endpoint");
        cb(Ok((sys, url, res)));
    });
}

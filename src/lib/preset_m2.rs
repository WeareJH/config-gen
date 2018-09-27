extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::{App, Error, HttpMessage, AsyncResponder};
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;
use preset::Preset;
use preset::ResourceDef;
use preset::RewriteFns;
use preset_m2_config_gen;
use preset_m2_opts::M2PresetOptions;
use regex::Regex;
use rewrites::RewriteContext;
use preset_m2_bundle_config::resolve_from_string;
use futures::{Future, Stream};
use preset_m2_requirejs_config::RequireJsMergedConfig;
use preset_m2_config_gen::Module;
use std::sync::Mutex;
use std::sync::Arc;
use preset_m2_requirejs_config::base_to_dirs;

///
/// The Magento 2 Preset
///
/// This contains some common middlewares and
/// resources specific to dealing with Magento 2 Websites
///
pub struct M2Preset {
    options: M2PresetOptions,
}

impl M2Preset {
    pub fn new(options: M2PresetOptions) -> M2Preset {
        M2Preset { options }
    }
    pub fn add_resources(&self, app: App<AppState>) -> App<AppState> {
        let resources: Vec<ResourceDef> = vec![
            (
                "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js",
                Method::GET,
                serve_instrumented_require_js,
            ),
            ("/__bs/reqs.json", Method::GET, serve_req_dump_json),
            ("/__bs/config.json", Method::GET, serve_config_dump_json),
            ("/__bs/loaders.json", Method::GET, serve_loaders_dump_json),
        ];

        let app = resources.into_iter().fold(app, |acc_app, (path, method, cb)| {
            acc_app.resource(&path, move |r| r.method(method).f(cb))
        });

        app.resource("/__bs/post", move |r| r.method(Method::POST).f(handle_post_data))
    }
}

///
/// Handle the requirejs post
///
fn handle_post_data(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {

    let a = req.state().require_merged_config.clone();

    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {

            let result: Result<RequireJsMergedConfig, serde_json::Error>
                = serde_json::from_str(std::str::from_utf8(&body).unwrap());
//
            let output = match result {
                Ok(next_config) => {
                    let mut mutex = a.lock().unwrap();
                    mutex.base_url = next_config.base_url;
                    mutex.deps     = next_config.deps;
                    mutex.map      = next_config.map;
                    mutex.config   = next_config.config;
                    mutex.paths    = next_config.paths;
                    mutex.shim     = next_config.shim;
                    "Was Good!"
                },
                Err(e) => "Was Bad!"
            };

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body("yo!"))
        })
        .responder()
}

///
/// The M2Preset adds some middleware, resources and
/// rewrites
///
impl Preset<AppState> for M2Preset {
    fn enhance(&self, app: App<AppState>) -> App<AppState> {
        self.add_resources(app)
    }
    fn rewrites(&self) -> RewriteFns {
        vec![replace_cookie_domain_on_page]
    }
    fn add_before_middleware(&self, app: App<AppState>) -> App<AppState> {
        app.middleware(ReqCatcher::new())
    }
}

///
/// This is the data type that is comes from each request
/// in a query param
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct ModuleData {
    pub url: String,
    pub id: String,
    pub referrer: String,
}

///
/// Extracting data means to look for a "bs_track" query
/// param, and then deserialize it's value (a JSON blob)
///
/// # Examples
///
/// ```
/// # use bs::preset_m2::*;
///
/// let data = r#"{
///   "url": "https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js",
///   "id": "Magento_Ui/js/form/form",
///   "referrer": "/"
/// }"#;
/// let d = extract_data(Some(&data.to_string())).unwrap();
///
/// assert_eq!(d, ModuleData {
///     url: String::from("https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js"),
///     id: String::from("Magento_Ui/js/form/form"),
///     referrer: String::from("/")
/// });
/// ```
///
pub fn extract_data(maybe_data: Option<&String>) -> Option<ModuleData> {
    maybe_data.and_then(|d| {
        let output = serde_json::from_str::<ModuleData>(&d);
        match output {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("oopS = {}", e);
                None
            }
        }
    })
}

pub struct ReqCatcher {}

impl ReqCatcher {
    pub fn new() -> ReqCatcher {
        ReqCatcher {}
    }
}

///
/// The ReqCatcher Middleware is responsible for checking if URLs
/// contain the bs_track payload, deserialising it's data and
/// then adding that data to the global vec of module data
///
impl Middleware<AppState> for ReqCatcher {
    /// This middleware handler will extract JSON blobs from URLS
    fn finish(&self, req: &HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        // try to convert some JSON into a valid ModuleData
        let module_data: Option<ModuleData> = extract_data(req.query().get("bs_track"));

        // We only care if we got a Some(ModuleData)
        // so we can use .map to unwrap & ignore the none;
        module_data.map(move |module_data| {
            // Get a reference to the Mutex wrapper
            let modules = &req.state().module_items;
            // acquire lock on the data so we can mutate it
            let mut data = modules.lock().unwrap();
            let mut exists = false;

            for d in data.iter() {
                if d == &module_data {
                    exists = true;
                }
            }

            if !exists {
                data.push(module_data);
            }
        });

        Finished::Done
    }
}

/// handler with path parameters like `/user/{name}/`
fn serve_instrumented_require_js(_req: &HttpRequest<AppState>) -> HttpResponse {
    let bytes = include_str!("./static/requirejs.js");

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(bytes)
}

/// serve a JSON dump of the current accumulated
fn serve_req_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let modules = &req.state().module_items;
    let modules = modules.lock().unwrap();

    let j = serde_json::to_string_pretty(&*modules).unwrap();

    HttpResponse::Ok().content_type("application/json").body(j)
}

/// serve a JSON dump of the current accumulated config
fn serve_loaders_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, modules)) => {
            let module_list = RequireJsMergedConfig::module_list(merged_config.mixins(), modules);
            Ok(module_list)
        },
        Err(e) => {
            Err("nah".to_string())
        }
    };

    match output {
        Ok(t) => {
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(t)
        }
        Err(e) => {
            HttpResponse::Ok()
                .content_type("text/plain")
                .body("NAH")
        }
    }
}

fn gather_state(req: &HttpRequest<AppState>) -> Result<(RequireJsMergedConfig, Vec<Module>), String> {
    let modules = &req.state()
        .module_items
        .lock()
        .expect("should lock & unwrap module_items");

    let merged_config = req.state()
        .require_merged_config
        .lock()
        .expect("should lock & unwrap require_merged_config");

    let maybe_opts = M2PresetOptions::get_opts(&req.state().program_config).expect("should clone program config");
    let bundle_path = maybe_opts.bundle_config;

    match bundle_path {
        Some(bun_config) => {
            match resolve_from_string(bun_config) {
                Ok(conf) => {
                    let modules = preset_m2_config_gen::run(modules.to_vec(), conf);
                    let mut next_config = (*merged_config).clone();

                    next_config.modules = Some(modules.clone());
                    next_config.optimize = next_config.optimize.or(Some("none".to_string()));
                    next_config.inline_text = next_config.inline_text.or(Some(true));
                    next_config.generate_source_maps = next_config.generate_source_maps.or(Some(true));

                    let dir = base_to_dirs(&next_config.base_url.expect("should access base_url")).expect("can create dirs");

                    next_config.base_url = Some(dir.base_url);
                    next_config.dir      = Some(dir.dir);

                    Ok((next_config, modules))
                },
                Err(e) => {
                    Err("Couldn't convert to string".to_string())
                }
            }
        },
        _ => Err("didnt match both".to_string())
    }
}

fn serve_config_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, modules)) => {
            match serde_json::to_string_pretty(&merged_config) {
                Ok(t) => Ok(t),
                Err(e) => {
                    Err("nah".to_string())
                }
            }
        },
        Err(e) => {
            Err("nah".to_string())
        }
    };

    match output {
        Ok(t) => {
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(t)
        }
        Err(e) => {
            HttpResponse::Ok()
                .content_type("text/plain")
                .body("NAH")
        }
    }
}

///
/// Remove an on-page cookie domain (usually in JSON blobs with Magento)
///
pub fn replace_cookie_domain_on_page(bytes: &str, context: &RewriteContext) -> String {
    let matcher = format!(r#""domain": ".{}","#, context.host_to_replace);
    Regex::new(&matcher)
        .unwrap()
        .replace_all(bytes, "")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_cookie_domain_on_page() {
        let bytes = r#"
        <script type="text/x-magento-init">
            {
                "*": {
                    "mage/cookies": {
                        "expires": null,
                        "path": "/",
                        "domain": ".www.acme.com",
                        "secure": false,
                        "lifetime": "10800"
                    }
                }
            }
        </script>
    "#;
        let replaced = replace_cookie_domain_on_page(
            &bytes,
            &RewriteContext {
                host_to_replace: String::from("www.acme.com"),
                target_host: String::from("127.0.0.1"),
                target_port: 80,
            },
        );
        println!("-> {}", replaced);
    }
}

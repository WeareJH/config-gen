extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{App, AsyncResponder, Error, HttpMessage};
use futures::{Future, Stream};
use preset::AppState;
use preset::Preset;
use preset::ResourceDef;
use preset::RewriteFns;
use preset_m2_bundle_config::resolve_from_string;
use preset_m2_config_gen;
use preset_m2_config_gen::Module;
use preset_m2_opts::M2PresetOptions;
use preset_m2_requirejs_config::base_to_dirs;
use preset_m2_requirejs_config::{RequireJsClientConfig};
use regex::Regex;
use rewrites::RewriteContext;
use from_file::FromFile;
use preset_m2_requirejs_config::RequireJsBuildConfig;
use proxy_transform::proxy_req_setup;
use actix_web::client::ClientResponse;
use proxy_transform::create_outgoing;
use proxy_transform::get_host_port;

type FutResp = Box<Future<Item = HttpResponse, Error = Error>>;

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
            ("/__bs/build.json", Method::GET, serve_build_json),
            ("/__bs/loaders.json", Method::GET, serve_loaders_dump_json),
            ("/__bs/seed.json", Method::GET, serve_seed_dump_json),
        ];

        let app = resources
            .into_iter()
            .fold(app, |acc_app, (path, method, cb)| {
                acc_app.resource(&path, move |r| r.method(method).f(cb))
            });

        app.resource(
            "/__bs/post",
            move |r| r.method(Method::POST).f(handle_post_data)
        ).resource(
            "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs-config.js",
            move |r| r.method(Method::GET).f(serve_requirejs_config)
        )
    }
}

///
/// Handle the requirejs post
///
fn handle_post_data(
    req: &HttpRequest<AppState>,
) -> FutResp {
    let a = req.state().require_client_config.clone();

    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let result: Result<RequireJsClientConfig, serde_json::Error> =
                serde_json::from_str(std::str::from_utf8(&body).unwrap());
            //
            match result {
                Ok(next_config) => {
                    let mut mutex = a.lock().unwrap();
                    mutex.base_url = next_config.base_url;
                    mutex.deps = next_config.deps;
                    mutex.map = next_config.map;
                    mutex.config = next_config.config;
                    mutex.paths = next_config.paths;
                    mutex.shim = next_config.shim;
                    "Was Good!".to_string()
                }
                Err(e) => e.to_string(),
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

fn serve_requirejs_config(original_request: &HttpRequest<AppState>) -> FutResp {
    apply_to_proxy_body(original_request, |mut b| {
        b.push_str(include_str!("./static/post_config.js"));
        b
    })
}

///
/// A helper for applying a transformation on a proxy
/// response before sending it back to the origin requester
///
fn apply_to_proxy_body<F>(original_request: &HttpRequest<AppState>, f: F) -> FutResp
    where F: Fn(String) -> String + 'static
{
    let mut outgoing = proxy_req_setup(original_request);
    let target_domain = original_request.state().opts.target.clone();
    let bind_port = original_request.state().opts.port;
    let (host, port) = get_host_port(original_request, bind_port);

    outgoing
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(move |proxy_response: ClientResponse| {
            proxy_response
                .body()
                .limit(1_000_000)
                .from_err()
                .and_then(move |body| {
                    use std::str;

                    let req_target = format!("{}:{}", host, port);
                    let body_content = str::from_utf8(&body[..]).unwrap();
                    let next_body: String = String::from(body_content);

                    Ok(create_outgoing(
                        &proxy_response.headers(),
                        target_domain.to_string(),
                        req_target,
                    ).body(f(next_body)))
                })
        })
        .responder()
}

#[derive(Serialize, Deserialize, Default)]
pub struct SeedData {
    pub client_config: RequireJsClientConfig,
    pub module_items: Vec<ModuleData>
}

impl FromFile for SeedData {}

/// serve a JSON dump of the current accumulated
fn serve_seed_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {

    let module_items = &req
        .state()
        .module_items
        .lock()
        .expect("should lock & unwrap module_items");

    let client_config = req
        .state()
        .require_client_config
        .lock()
        .expect("should lock & unwrap require_client_config");

    let output = SeedData { client_config: client_config.clone(), module_items: module_items.to_vec() };

    let output = match serde_json::to_string_pretty(&output) {
        Ok(t) => Ok(t),
        Err(e) => Err(e.to_string())
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(e) => HttpResponse::Ok().content_type("application/json").body(e),
    }
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
            let module_list = RequireJsClientConfig::module_list(
                RequireJsClientConfig::mixins(&merged_config.config)
            , modules);
            Ok(module_list)
        }
        Err(e) => Err(e),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("text/plain").body(t),
        Err(_e) => HttpResponse::Ok().content_type("text/plain").body("NAH"),
    }
}

fn gather_state(
    req: &HttpRequest<AppState>,
) -> Result<(RequireJsBuildConfig, Vec<Module>), String> {
    let modules = &req
        .state()
        .module_items
        .lock()
        .expect("should lock & unwrap module_items");

    let client_config = req
        .state()
        .require_client_config
        .lock()
        .expect("should lock & unwrap require_client_config");

    let maybe_opts = M2PresetOptions::get_opts(&req.state().program_config)
        .expect("should clone program config");
    let bundle_path = maybe_opts.bundle_config;

    match bundle_path {
        Some(bun_config) => match resolve_from_string(bun_config) {
            Ok(conf) => {
                let modules = preset_m2_config_gen::run(modules.to_vec(), conf);
                let mut derived_build_config = RequireJsBuildConfig::default();

                derived_build_config.modules = Some(modules.clone());
                derived_build_config.deps = client_config.deps.clone();
                derived_build_config.map = client_config.map.clone();
                derived_build_config.config = client_config.config.clone();
                derived_build_config.shim = client_config.shim.clone();
                derived_build_config.paths = client_config.paths.clone();

                let base_url = client_config.base_url.clone().unwrap_or("".to_string());

                match base_to_dirs(&base_url) {
                    Ok(dir) => {
                        derived_build_config.base_url = Some(dir.base_url);
                        derived_build_config.dir = Some(dir.dir);
                    }
                    Err(_e) => {
                        eprintln!("Could not use base_url to create baseUrl + dir");
                    }
                }

                Ok((derived_build_config, modules))
            }
            Err(_e) => Err("Couldn't convert to string".to_string()),
        },
        _ => Err("didnt match both".to_string()),
    }
}

fn serve_config_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match req.state().require_client_config.lock() {
        Ok(config) => match serde_json::to_string_pretty(&*config) {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string())
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(_e) => HttpResponse::Ok().content_type("application/json").body("Could not serve config"),
    }
}

fn serve_build_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let output = match gather_state(req) {
        Ok((merged_config, _)) => match serde_json::to_string_pretty(&merged_config) {
            Ok(t) => Ok(t),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    };

    match output {
        Ok(t) => HttpResponse::Ok().content_type("application/json").body(t),
        Err(e) => HttpResponse::Ok().content_type("application/json").body(e.to_string()),
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

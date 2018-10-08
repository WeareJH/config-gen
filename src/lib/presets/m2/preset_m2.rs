extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::{App, Error, HttpRequest, HttpResponse};
use futures::Future;

use from_file::FromFile;
use preset::{AppState, Preset, ResourceDef, RewriteFns};

use super::bundle_config::BundleConfig;
use super::bundle_config::Module;
use super::config_gen;
use super::handlers;
use super::opts::M2PresetOptions;
use super::replace_cookie_domain;
use super::requirejs_config::{RequireJsBuildConfig, RequireJsClientConfig};
use preset::AsyncResourceDef;

pub type FutResp = Box<Future<Item = HttpResponse, Error = Error>>;

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
}

const PATH_REQUIRE_JS: &'static str =
    "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js";
const PATH_REQUIRE_CNF: &'static str =
    "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs-config.js";
const PATH_CONF_POST: &'static str = "/__bs/post";

///
/// The M2Preset adds some middleware, resources and
/// rewrites
///
impl Preset<AppState> for M2Preset {
    ///
    /// This will add the bulk of the API endpoint for
    /// all the functionality related to the M2 Preset
    ///
    fn enhance(&self, app: App<AppState>) -> App<AppState> {
        //
        // Http Responders are handlers that return synchronously
        // which is suitable for most routes.
        //
        let http_responders: Vec<ResourceDef> = vec![
            (PATH_REQUIRE_JS, Method::GET, handlers::serve_r_js::handle),
            ("/__bs/reqs.json", Method::GET, handlers::requests::handle),
            ("/__bs/config.json", Method::GET, handlers::config::handle),
            ("/__bs/build.json", Method::GET, handlers::build::handle),
            ("/__bs/loaders.json", Method::GET, handlers::loaders::handle),
            ("/__bs/seed.json", Method::GET, handlers::seed::handle),
        ];

        //
        // Async Responders are needed when there's additional
        // work to be done in a handler.
        //
        let http_async_responders: Vec<AsyncResourceDef> = vec![
            (PATH_CONF_POST, Method::POST, handlers::config_post::handle),
            (
                PATH_REQUIRE_CNF,
                Method::GET,
                handlers::config_capture::handle,
            ),
        ];

        let app = http_responders
            .into_iter()
            .fold(app, |acc_app, (path, method, cb)| {
                acc_app.resource(&path, move |r| r.method(method).f(cb))
            });

        http_async_responders
            .into_iter()
            .fold(app, |acc_app, (path, method, cb)| {
                acc_app.resource(&path, move |r| r.method(method).f(cb))
            })
    }
    ///
    /// The only rewrite that the M2 preset uses
    /// is one to remove the cookie-domain on the page
    /// as it prevents session-based actions.
    ///
    fn rewrites(&self) -> RewriteFns {
        vec![replace_cookie_domain::rewrite]
    }
    ///
    /// a 'before' middleware is used to track incoming requests that contain
    /// the metadata needed to build up the tracking.
    ///
    fn add_before_middleware(&self, app: App<AppState>) -> App<AppState> {
        app.middleware(handlers::req_capture::ReqCapture::new())
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

#[derive(Serialize, Deserialize, Default)]
pub struct SeedData {
    pub client_config: RequireJsClientConfig,
    pub module_items: Vec<ModuleData>,
}

impl FromFile for SeedData {}

pub fn gather_state(
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
        Some(bun_config_path) => match BundleConfig::from_yml_file(&bun_config_path) {
            Ok(bundle_config) => {
                let module_blacklist = bundle_config.module_blacklist.clone().unwrap_or(vec![]);
                let mut blacklist = vec!["js-translation".to_string()];
                blacklist.extend(module_blacklist);

                let filtered =
                    RequireJsBuildConfig::drop_blacklisted(&modules.to_vec(), &blacklist);
                let bundle_modules = config_gen::generate_modules(filtered, bundle_config);
                let mut derived_build_config = RequireJsBuildConfig::default();

                derived_build_config.deps = client_config.deps.clone();
                derived_build_config.map = client_config.map.clone();
                derived_build_config.config = client_config.config.clone();

                let mut c = client_config.paths.clone();
                derived_build_config.paths = RequireJsBuildConfig::strip_paths(&c);

                let mut shims = client_config.shim.clone();

                {
                    RequireJsBuildConfig::fix_shims(&mut shims);
                }

                derived_build_config.shim = shims;

                derived_build_config.modules = Some(bundle_modules.clone());

                Ok((derived_build_config, bundle_modules))
            }
            Err(e) => Err(e.to_string()),
        },
        _ => Err("didnt match both".to_string()),
    }
}

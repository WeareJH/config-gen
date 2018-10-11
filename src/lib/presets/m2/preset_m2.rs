extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::{App, Error, HttpResponse};
use futures::Future;

use preset::{AppState, Preset, ResourceDef, RewriteFns};

use super::handlers;
use super::opts::M2PresetOptions;
use super::replace_cookie_domain;
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
            ("/__bs/loaders.js", Method::GET, handlers::loaders::handle),
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
            .fold(app, |acc_app, (path, _method, handle)| {
                acc_app.resource(&path, move |r| r.f(handle))
            });

        http_async_responders
            .into_iter()
            .fold(app, |acc_app, (path, method, handle)| {
                acc_app.resource(&path, move |r| r.f(handle))
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

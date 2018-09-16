extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use preset::AppState;
use preset::Preset;
use preset::ResourceDef;
use preset::RewriteFns;
use preset_m2_config_gen;
use preset_m2_config_gen::ConfigItems;
use regex::Regex;
use rewrites::RewriteContext;
use url::Url;

///
/// The Magento 2 Preset
///
/// This contains some common middlewares and
/// resources specific to dealing with Magento 2 Websites
///
pub struct M2Preset {}

impl M2Preset {
    pub fn new() -> M2Preset {
        M2Preset {}
    }
    pub fn add_resources(&self, app: App<AppState>) -> App<AppState> {
        let resources: Vec<ResourceDef> = vec![
            (
                "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js",
                serve_instrumented_require_js,
            ),
            ("/__bs/reqs.json", serve_req_dump_json),
            ("/__bs/config.json", serve_config_dump_json),
        ];
        resources.into_iter().fold(app, |acc_app, (path, cb)| {
            acc_app.resource(&path, move |r| r.method(Method::GET).f(cb))
        })
    }

    pub fn add_before_middleware(&self, app: App<AppState>) -> App<AppState> {
        app.middleware(ReqCatcher::new())
    }
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
/// let url = "https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js?bs_track=%7B%22url%22%3A%22https%3A%2F%2F127.0.0.1%3A8080%2Fstatic%2Fversion1536567404%2Ffrontend%2FAcme%2Fdefault%2Fen_GB%2FMagento_Ui%2Fjs%2Fform%2Fform.js%22%2C%22id%22%3A%22Magento_Ui%2Fjs%2Fform%2Fform%22%2C%22referrer%22%3A%22%2F%22%7D";
/// let d = extract_data(url).unwrap();
///
/// assert_eq!(d, ModuleData {
///     url: String::from("https://127.0.0.1:8080/static/version1536567404/frontend/Acme/default/en_GB/Magento_Ui/js/form/form.js"),
///     id: String::from("Magento_Ui/js/form/form"),
///     referrer: String::from("/")
/// });
/// ```
///
pub fn extract_data(url: &str) -> Option<ModuleData> {
    let url = Url::parse(url).ok()?;

    let matched = url
        .query_pairs()
        .find(|(key, _)| key == "bs_track")
        .map(|(_, value)| value)?;

    let d: Result<ModuleData, _> = serde_json::from_str(&matched);

    match d {
        Ok(data) => Some(data),
        Err(e) => {
            println!("{:?}", e);
            None
        }
    }
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
        let module_data: Option<ModuleData> = extract_data(&req.uri().to_string());

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
fn serve_config_dump_json(req: &HttpRequest<AppState>) -> HttpResponse {
    let modules = &req.state().module_items;
    let mut modules = modules.lock().unwrap();
    let c: ConfigItems = r#"
    [
      {
        "name": "requirejs/require",
        "urls": [
          "/",
          "/nav/new-in.html"
        ],
        "children": [
          {
            "name": "bundles/product",
            "urls": [
              "/wellbeing-essential-oil-blends-collection.html"
            ],
            "children": [

            ]
          }
        ]
      }
    ]
    "#.into();

    let config_as_string = preset_m2_config_gen::run(modules.to_vec(), c);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(config_as_string)
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

extern crate serde;
extern crate serde_json;

use actix_web::http::Method;
use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use options::ProxyOpts;
use preset::AppState;
use preset::Preset;
use preset::RewriteFns;
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
        let resources = vec![(
            "/static/{version}/frontend/{vendor}/{theme}/{locale}/requirejs/require.js",
            serve_instrumented_require_js,
        )];
        resources.into_iter().fold(app, |acc_app, (path, cb)| {
            acc_app.resource(&path, move |r| r.method(Method::GET).f(cb))
        })
    }
}

///
/// The M2Preset adds some middleware, resources and
/// rewrites
///
impl Preset<AppState> for M2Preset {
    fn enhance(&self, app: App<AppState>) -> App<AppState> {
        let app = app.middleware(ReqCatcher::new());
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
#[derive(Debug, Deserialize, PartialEq)]
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

impl Middleware<AppState> for ReqCatcher {
    fn finish(&self, req: &HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        let md: Option<ModuleData> = extract_data(&req.uri().to_string());
        md.map(|module_data| {
            println!("module_id: {}", module_data.id);
        });
        Finished::Done
    }
}

/// handler with path parameters like `/user/{name}/`
fn serve_instrumented_require_js(req: &HttpRequest<AppState>) -> HttpResponse {
    let bytes = include_str!("./static/requirejs.js");

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(bytes)
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

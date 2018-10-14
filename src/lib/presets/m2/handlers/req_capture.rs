use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::{HttpRequest, HttpResponse};
use preset::AppState;
use presets::m2::module_meta_data::ModuleData;
use serde_json;

pub struct ReqCapture {}

impl ReqCapture {
    pub fn new() -> ReqCapture {
        ReqCapture {}
    }
}

///
/// The ReqCatcher Middleware is responsible for checking if URLs
/// contain the bs_track payload, deserialising it's data and
/// then adding that data to the global vec of module data
///
impl Middleware<AppState> for ReqCapture {
    /// This middleware handler will extract JSON blobs from URLS
    fn finish(&self, req: &HttpRequest<AppState>, _resp: &HttpResponse) -> Finished {
        // try to convert some JSON into a valid ModuleData
        let module_data: Option<ModuleData> = extract_data(req.query().get("bs_track"));

        // We only care if we got a Some(ModuleData)
        // so we can use .map to unwrap & ignore the none;
        module_data.map(move |module_data| {
            // Get a reference to the Mutex wrapper
            let modules = &req.state().req_log;
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

///
/// Extracting data means to look for a "bs_track" query
/// param, and then deserialize it's value (a JSON blob)
///
/// # Examples
///
/// ```
/// # use bs::presets::m2::handlers::req_capture::*;
/// # use bs::presets::m2::module_meta_data::*;
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

use actix_web::http::Method;
use actix_web::{App, HttpRequest, HttpResponse};
use app_state::AppState;
use presets::m2::preset_m2::FutResp;
use rewrites::RewriteContext;
use std::fmt;
use serde_json;

pub trait Preset<T> {
    fn enhance(&self, app: App<T>) -> App<T>;
    fn rewrites(&self) -> RewriteFns {
        vec![]
    }
    fn add_before_middleware(&self, app: App<T>) -> App<T> {
        app
    }
    fn add_after_middleware(&self, app: App<T>) -> App<T> {
        app
    }
}

///
/// Presets have the chance to validate their own options
///
pub trait PresetOptions {
    fn validate(_options: serde_json::Value) -> Result<(), PresetError> {
        Ok(())
    }
}

///
/// The following are just aliases
///
pub type RewriteFns = Vec<fn(&str, &RewriteContext) -> String>;
pub type ResourceDef = (String, Method, fn(&HttpRequest<AppState>) -> HttpResponse);
pub type AsyncResourceDef = (String, Method, fn(&HttpRequest<AppState>) -> FutResp);

pub enum PresetError {
    ValidationFailed(String),
}

impl fmt::Display for PresetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PresetError::ValidationFailed(s) => write!(f, "{}", s),
        }
    }
}

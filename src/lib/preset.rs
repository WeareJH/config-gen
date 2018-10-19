use actix_web::http::Method;
use actix_web::{App, HttpRequest, HttpResponse};
use app_state::AppState;
use presets::m2::preset_m2::FutResp;
use rewrites::RewriteContext;

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
/// The following are just aliases
///
pub type RewriteFns = Vec<fn(&str, &RewriteContext) -> String>;
pub type ResourceDef<'a> = (&'a str, Method, fn(&HttpRequest<AppState>) -> HttpResponse);
pub type AsyncResourceDef<'a> = (&'a str, Method, fn(&HttpRequest<AppState>) -> FutResp);

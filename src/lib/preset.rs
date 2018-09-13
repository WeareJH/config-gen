use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use options::ProxyOpts;
use rewrites::RewriteContext;

pub trait Preset<T> {
    fn enhance(&self, app: App<T>) -> App<T>;
    fn rewrites(&self) -> RewriteFns {
        vec![]
    }
}
pub type RewriteFns = Vec<fn(&str, &RewriteContext) -> String>;
pub type Resource = (String, fn(&HttpRequest<AppState>) -> HttpResponse);

#[derive(Default)]
pub struct AppState {
    pub opts: ProxyOpts,
    pub rewrites: RewriteFns,
}

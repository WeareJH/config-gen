use actix_web::middleware::Finished;
use actix_web::middleware::Middleware;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use options::ProxyOpts;

pub trait Preset<T> {
    fn resources(&self) -> Vec<(String, fn(&HttpRequest<T>) -> HttpResponse)> {
        vec![]
    }
    fn before_middleware(&self) -> Vec<Box<Middleware<T>>> {
        vec![]
    }
    fn after_middleware(&self) -> Vec<Box<Middleware<T>>> {
        vec![]
    }
}

pub type Resource = (String, fn(&HttpRequest<ProxyOpts>) -> HttpResponse);
